use std::iter::repeat_n;

use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BF {
    Dbg(&'static str),
    Left,   // `<`
    Right,  // `>`
    Inc,    // `+`
    Dec,    // `-`
    Input,  // `,`
    Output, // `.`
    LBrac,  // `[`
    RBrac,  // `]`
}

impl BF {
    pub fn parse(code: &str) -> Vec<BF> {
        let mut out = vec![];
        for c in code.chars() {
            use BF::*;
            let inst = match c {
                '<' => Left,
                '>' => Right,
                '+' => Inc,
                '-' => Dec,
                ',' => Input,
                '.' => Output,
                '[' => LBrac,
                ']' => RBrac,
                _ => continue,
            };
            out.push(inst);
        }
        out
    }

    pub fn show(self) -> char {
        use BF::*;
        match self {
            Left => '<',
            Right => '>',
            Inc => '+',
            Dec => '-',
            Input => ',',
            Output => '.',
            LBrac => '[',
            RBrac => ']',
            _ => unimplemented!(),
        }
    }
}

pub fn show_bf(code: &[BF], dbg_info: bool) -> String {
    let mut s = String::new();

    for i in code {
        if let BF::Dbg(c) = i {
            if dbg_info {
                s += &format!("\n`{}`: ", c);
            }
            continue;
        }
        s.push(i.show());
    }

    s
}

pub fn translate(stack: &[StackInst]) -> Vec<BF> {
    let mut stack = Vec::from(stack);
    StackInst::expand(&mut stack);

    let mut bf = vec![];

    // Allocate extra cell (in case stack is empty), then goto 1
    bf.extend(BF::parse(">+[>"));

    for inst in stack {
        use StackInst::*;
        use BF::*;

        match inst {
            Push(b) => {
                bf.push(Right);
                bf.extend(repeat_n(Inc, b as _));
            }
            Swap => bf.extend(BF::parse(
                "
                <[->>+<<] // Move 1 into 3
                >[-<+>]   // Shift 2 into 1
                >[-<+>]   // Shift 3 into 2
                <         // Point back at 2
                ",
            )),

            Copy => bf.extend(BF::parse("[->+>+<<]>>[-<<+>>]<")),
            Mul => bf.extend(BF::parse(
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
            Add => bf.extend(BF::parse("[-<+>]<")),
            Sub => bf.extend(BF::parse("[-<->]<")),
            Alloc(n) => bf.extend(repeat_n(Right, n as _)),
            Dealloc(n) => {
                for _ in 0..n {
                    bf.extend(BF::parse("[-]<"));
                }
            }
            LclRead(n) => {
                let left = repeat_n(Left, n).collect::<Vec<_>>();
                let right = repeat_n(Right, n).collect::<Vec<_>>();
                bf.extend(&left);
                bf.extend(BF::parse("[-"));
                bf.extend(&right);
                bf.extend(BF::parse(">+>+<<")); // Make 2 copies
                bf.extend(&left);
                bf.extend(BF::parse("]"));
                bf.extend(&right);
                bf.extend(BF::parse(">>[-<<")); // Move 1 copy back
                bf.extend(&left);
                bf.extend(BF::parse("+"));
                bf.extend(&right);
                bf.extend(BF::parse(">>]<"))
            }
            LclStr(n) => {
                let left = repeat_n(Left, n).collect::<Vec<_>>();
                let right = repeat_n(Right, n).collect::<Vec<_>>();
                bf.extend(&left);
                bf.extend(BF::parse("[-]")); // Erase previous value
                bf.extend(&right);
                bf.extend(BF::parse("[-")); // Enter move loop
                bf.extend(&left);
                bf.extend(BF::parse("+")); // Shift 1 unit over
                bf.extend(&right);
                bf.extend(BF::parse("]<")); // Exit loop and move stack head
            }
            Label(n) if n != 0 => {
                bf.extend(BF::parse("<[->+>+<<]>>[-<<+>>]<")); // Move to, and then copy, label
                bf.extend(repeat_n(Dec, n as _)); // Check equality
                bf.extend(BF::parse(
                    "
                    >+<      // Push 1
                    [[-]>-<] // If unequal then remove 1
                    >[-<+>]< // Move 1 if (label == n) else leave 0
                    ",
                ));
                bf.extend(BF::parse("[-<[-]<"));
                // Enter block if label at head
                // Then, discard equality check and label, and point to stack
            }
            Neq => bf.extend(BF::parse("[-<->]<")), // Check equality
            LNot => bf.extend(BF::parse(
                "
                >+<      // Place 1
                [[-]>-<] // If nonzero then erase 1
                >[-<+>]< // Move 1 (or 0)
                ",
            )),
            GrEq => bf.extend(BF::parse(
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
            LAnd => bf.extend(BF::parse(
                "
                >++<            // Place 2
                [[-]>-<]<       // Subtract 1 if rhs is nonzero
                [[-]>>-<<]      // Subtract 1 if lhs is nonzero
                >>[-[-<<+>>]]<< // Return result
                ",
            )),
            LOr => bf.extend(BF::parse(
                "
                [[-]>+<]<
                [[-]>>+<<]
                >>[[-]<<+>>]<<
                ",
            )),
            Xor => bf.extend(BF::parse(
                "
                // Bitwise Sum
                >>+>+>+>+>+>+>+>+>+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                <[->+<]>
                >>[>]+>+>+>+>+>+>+>+>+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                >>[>]++++++++++++++++
                [-<[-<<<<<<<<<<<<<<<<+>>>>>>>>>>>>>>>>]>[-<+>]<]
                // Condense into 1 cell
                <[<]<+>>[>]
                <[>+<---[[-]>-<]>[-<<[<]<[-<+>>+<]>[-<+>]>[>]>]<<[<]<[->++<]>[-<+>]>>[>]<]<<<
                ",
            )),
            And => bf.extend(BF::parse(
                "
                // Bitwise Sum
                >>+>+>+>+>+>+>+>+>+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                <[->+<]>
                >>[>]+>+>+>+>+>+>+>+>+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                >>[>]++++++++++++++++
                [-<[-<<<<<<<<<<<<<<<<+>>>>>>>>>>>>>>>>]>[-<+>]<]
                // Condense into 1 cell
                <[<]<+>>[>]
                <[>+<----[[-]>-<]>[-<<[<]<[-<+>>+<]>[-<+>]>[>]>]<<[<]<[->++<]>[-<+>]>>[>]<]<<<
                ",
            )),
            Or => bf.extend(BF::parse(
                // Implemented as NOR, then Bitwise negation
                "
                // Bitwise Sum
                >>+>+>+>+>+>+>+>+>+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                <[->+<]>
                >>[>]+>+>+>+>+>+>+>+>+>+>+>+>+>+>+>+[<]<
                [->>[>]<[--[++++[->]>]++<]>--<<[<]<]
                >>[>]++++++++++++++++
                [-<[-<<<<<<<<<<<<<<<<+>>>>>>>>>>>>>>>>]>[-<+>]<]
                // Condense into 1 cell
                <[<]<+>>[>]
                <[>+<--[[-]>-<]>[-<<[<]<[-<+>>+<]>[-<+>]>[>]>]<<[<]<[->++<]>[-<+>]>>[>]<]<<<
                // Bitwise negation                
                [->-<]>-[-<+>]<
                ",
            )),
            Not => bf.extend(BF::parse("[->-<]>-[-<+>]<")), // Inverse of 2's complement
            Negate => bf.extend(BF::parse("[->-<]>[-<+>]<")),
            LShift => bf.extend(BF::parse("[-<[->>+>+<<<]>>[-<<+>>]>[-<<<+>>>]<<]<")),
            RShift => bf.extend(BF::parse(
                "
                <[->>+<<]> // swap x & y
                [->+>+<[-[-[>+>]>[>>]<]>[>>]<<<]>-[-<+>]<<] // CBA to explain
                >[-<<+>>]<<
                ",
            )),
            Div => bf.extend(BF::parse(
                // From https://esolangs.org/wiki/Brainfuck_algorithms#Divmod
                "
                // Execute
                <[->-[>+>>]>[+[-<+>]>+>>]<<<<<]
                // Return value
                >[-]>[-]>[-<<<+>>>]<<<
                ",
            )),
            Mod => bf.extend(BF::parse(
                // From https://esolangs.org/wiki/Brainfuck_algorithms#Modulo
                "
                // Prepare state
                [->>+<<]<[->+<]>
                // Perform modulo
                [>+>->+<[>]>[<+>-]<<[<]>-]
                // Return answer
                >[-]>[-]>[-<<<<+>>>>]<<<<
                ",
            )),

            StkRead => {
                // Prepare state
                bf.extend(BF::parse("-[->+>+<<]>>>"));

                // Taken from the internet: https://www.inshame.com/2008/02/efficient-brainfuck-tables.html (directions flipped)
                bf.extend(BF::parse(
                    "
                    <[<<<[->>>>+<<<<]>>[-<+>]>[-<+>]<-]
                    <<<[->+>>+<<<]>>>[-<<<+>>>]<
                    [[->+<]<[->+<]>>>>[-<<<<+>>>>]<<-]>>
                    ",
                ));

                // Move stack head back
                bf.extend(BF::parse("<<<"))
            }

            StkStr => {
                // Prepare state
                bf.extend(BF::parse("-[->+>+<<]>>[-<<+>>]"));

                // Taken from the sample place
                bf.extend(BF::parse(
                    "
                    <[<<<[->>>>+<<<<]>[-<+>]>[-<+>]>[-<+>]<-]
                    <<<[-]>[-<+>]>
                    [[->+<]>>>[-<<<<+>>>>]<<-]>>
                    ",
                ));

                // Move stack head back
                bf.extend(BF::parse("<<<<"));
            }

            Branch(t, f) => {
                bf.push(Right);
                bf.extend(repeat_n(Inc, f as _));
                bf.push(Left);
                bf.extend(BF::parse("[[-]>"));
                if t >= f {
                    bf.extend(repeat_n(Inc, t.wrapping_sub(f) as _));
                } else {
                    bf.extend(repeat_n(Dec, f.wrapping_sub(t) as _));
                }
                bf.extend(BF::parse("<]>[-<+>]<"));
                bf.extend(BF::parse(">]"));
            }
            Goto => bf.extend(BF::parse(">]")),
            PutChar => bf.extend(BF::parse(".[-]<")),
            Label(0) | Nop | Debug(_) | Comment(_) => {}
            i => todo!("{:?}", i),
        }
    }

    bf.extend(BF::parse("<]"));

    bf
}
