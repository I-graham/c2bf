use std::iter::repeat_n;

use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BFInst {
    Dbg(&'static str),
    Left,  // `<`
    Right, // `>`
    Inc,   // `+`
    Dec,   // `-`
    In,    // `,`
    Out,   // `.`
    LBrac, // `[`
    RBrac, // `]`
}

impl BFInst {
    pub fn parse(code: &str) -> Vec<BFInst> {
        let mut out = vec![];
        for c in code.chars() {
            use BFInst::*;
            let inst = match c {
                '<' => Left,
                '>' => Right,
                '+' => Inc,
                '-' => Dec,
                ',' => In,
                '.' => Out,
                '[' => LBrac,
                ']' => RBrac,
                _ => continue,
            };
            out.push(inst);
        }
        out
    }

    pub fn show(self) -> char {
        use BFInst::*;
        match self {
            Left => '<',
            Right => '>',
            Inc => '+',
            Dec => '-',
            In => ',',
            Out => '.',
            LBrac => '[',
            RBrac => ']',
            _ => unimplemented!(),
        }
    }

    pub fn show_code(code: &[BFInst], dbg_info: bool) -> String {
        let mut s = String::new();

        for i in code {
            if let BFInst::Dbg(c) = i {
                if dbg_info {
                    s += &format!("\n`{}`: ", c);
                }
                continue;
            }
            s.push(i.show());
        }

        s
    }
}

pub fn asm_to_bf(stack: &[StackInst]) -> Vec<BFInst> {
    let mut stack = Vec::from(stack);
    StackInst::expand(&mut stack);

    let mut bf = vec![];

    // Allocate extra cell (in case stack is empty), then goto 1
    bf.extend(BFInst::parse(">+[>"));

    for inst in stack {
        use BFInst::*;
        use StackInst::*;

        bf.push(Dbg(format!("{:?}", inst).leak()));

        match inst {
            PushB(b) => {
                bf.push(Right);
                bf.extend(repeat_n(Inc, b as _));
            }
            DiscardB => {
                bf.extend(BFInst::parse("[-]<"));
            }
            SwapB => bf.extend(BFInst::parse(
                "
                <[->>+<<] // Move 1 into 3
                >[-<+>]   // Shift 2 into 1
                >[-<+>]   // Shift 3 into 2
                <         // Point back at 2
                ",
            )),

            CopyB => bf.extend(BFInst::parse("[->+>+<<]>>[-<<+>>]<")),
            MulB => bf.extend(BFInst::parse(
                "
                <[->>+<<]        // Make room for return value
                >[-              // repeat x times
                   >[->+>+<<]    // copy y to 2 new stack locations
                   >>[-<<+>>]    // Use one of these copies to replace y
                   <[-<<<+>>>]   // Add other one of these to the return value
                   <<            // Point back at x
                ]
                >[-]<<           // clear y & point at x
                ",
            )),
            AddB => bf.extend(BFInst::parse("[-<+>]<")),
            SubB => bf.extend(BFInst::parse("[-<->]<")),
            Alloc(n) => bf.extend(repeat_n(Right, n as _)),
            Dealloc(n) => {
                for _ in 0..n {
                    bf.extend(BFInst::parse("[-]<"));
                }
            }
            LclReadB(n) => {
                let left = repeat_n(Left, n).collect::<Vec<_>>();
                let right = repeat_n(Right, n).collect::<Vec<_>>();
                bf.extend(&left);
                bf.extend(BFInst::parse("[-"));
                bf.extend(&right);
                bf.extend(BFInst::parse(">+>+<<")); // Make 2 copies
                bf.extend(&left);
                bf.extend(BFInst::parse("]"));
                bf.extend(&right);
                bf.extend(BFInst::parse(">>[-<<")); // Move 1 copy back
                bf.extend(&left);
                bf.extend(BFInst::parse("+"));
                bf.extend(&right);
                bf.extend(BFInst::parse(">>]<"))
            }
            LclStrB(n) => {
                let left = repeat_n(Left, n).collect::<Vec<_>>();
                let right = repeat_n(Right, n).collect::<Vec<_>>();
                bf.extend(&left);
                bf.extend(BFInst::parse("[-]")); // Erase previous value
                bf.extend(&right);
                bf.extend(BFInst::parse("[-")); // Enter move loop
                bf.extend(&left);
                bf.extend(BFInst::parse("+")); // Shift 1 unit over
                bf.extend(&right);
                bf.extend(BFInst::parse("]<")); // Exit loop and move stack head
            }
            Label(n) if n != 0 => {
                bf.extend(BFInst::parse("<[->+>+<<]>>[-<<+>>]<")); // Move to, and then copy, label
                bf.extend(repeat_n(Dec, n as _)); // Check equality
                bf.extend(BFInst::parse(
                    "
                    >+<      // Push 1
                    [[-]>-<] // If unequal then remove 1
                    >[-<+>]< // Move 1 if (label == n) else leave 0
                    ",
                ));
                bf.extend(BFInst::parse("[-<[-]<"));
                // Enter block if label at head
                // Then, discard equality check and label, and point to stack
            }
            Neq => bf.extend(BFInst::parse("[-<->]<")), // Check equality
            LNot => bf.extend(BFInst::parse(
                "
                >+<      // Place 1
                [[-]>-<] // If nonzero then erase 1
                >[-<+>]< // Move 1 (or 0)
                ",
            )),
            GrEq => bf.extend(BFInst::parse(
                // Memory layout: y x
                // Return value: nonzero iff x < y
                // Note: if x==0 then we can just return y
                // If not we use the loop to decrement each repeatedly until one is 0
                "
                >+<                     // return value = true
                [                       // while x != 0
                    <[>]                // point to y then split based on if y=0
                    >[<+>[-]+>[-]>>]<<< // If y=0 then set x=y=1 & clear return value
                    -<->                // Decrement x & y                    
                ]
                <[-]                    // clear y
                >>[-<<+>>]<<            // Push return value
                ",
            )),
            LAnd => bf.extend(BFInst::parse(
                "
                >++<            // Place 2
                [[-]>-<]<       // Subtract 1 if rhs is nonzero
                [[-]>>-<<]      // Subtract 1 if lhs is nonzero
                >>[-[-<<+>>]]<< // Return result
                ",
            )),
            LOr => bf.extend(BFInst::parse(
                "
                [[-]>+<]<
                [[-]>>+<<]
                >>[[-]<<+>>]<<
                ",
            )),
            Xor => bf.extend(BFInst::parse(
                "
                // Bitwise Sum
                >>+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                <[->+<]>
                >>[>]+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                >>[>]++++++++
                [-<[-<<<<<<<<+>>>>>>>>]>[-<+>]<]
                // Condense into 1 cell
                <[<]<+>>[>]
                <[>+<---[[-]>-<]>[-<<[<]<[-<+>>+<]>[-<+>]>[>]>]<<[<]<[->++<]>[-<+>]>>[>]<]<<<
                ",
            )),
            And => bf.extend(BFInst::parse(
                "
                // Bitwise Sum
                >>+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                <[->+<]>
                >>[>]+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                >>[>]++++++++
                [-<[-<<<<<<<<+>>>>>>>>]>[-<+>]<]
                // Condense into 1 cell
                <[<]<+>>[>]
                <[>+<----[[-]>-<]>[-<<[<]<[-<+>>+<]>[-<+>]>[>]>]<<[<]<[->++<]>[-<+>]>>[>]<]<<<
                ",
            )),
            Or => bf.extend(BFInst::parse(
                // Implemented as NOR, then Bitwise negation
                "
                // Bitwise Sum
                >>+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                <[->+<]>
                >>[>]+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                >>[>]++++++++
                [-<[-<<<<<<<<+>>>>>>>>]>[-<+>]<]
                // Condense into 1 cell
                <[<]<+>>[>]
                <[>+<--[[-]>-<]>[-<<[<]<[-<+>>+<]>[-<+>]>[>]>]<<[<]<[->++<]>[-<+>]>>[>]<]<<<
                // Bitwise negation                
                [->-<]>-[-<+>]<
                ",
            )),
            Not => bf.extend(BFInst::parse("[->-<]>-[-<+>]<")), // Inverse of 2's complement
            LShift => bf.extend(BFInst::parse("[-<[->>+>+<<<]>>[-<<+>>]>[-<<<+>>>]<<]<")),
            RShift => bf.extend(BFInst::parse(
                "
                <[->>+<<]> // swap x & y
                [->+>+<[-[-[>+>]>[>>]<]>[>>]<<<]>-[-<+>]<<] // CBA to explain
                >[-<<+>>]<<
                ",
            )),

            Branch(t, f) => {
                bf.push(Right);
                bf.extend(repeat_n(Inc, f as _));
                bf.push(Left);
                bf.extend(BFInst::parse("[[-]>"));
                bf.extend(repeat_n(Inc, t.wrapping_sub(f) as _));
                bf.extend(BFInst::parse("<]>[-<+>]<"));
                bf.extend(BFInst::parse(">]"));
            }
            Goto => bf.extend(BFInst::parse(">]")),
            PrintChar => bf.extend(BFInst::parse(".[-]<")),
            Label(0) | Nop | Debug(_) | Comment(_) => {}
            i => todo!("{:?}", i),
        }
    }

    bf.extend(BFInst::parse("<]"));

    bf
}
