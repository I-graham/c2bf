use std::iter::repeat_n;

use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BFInst {
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
            Right => '<',
            Inc => '+',
            Dec => '-',
            In => ',',
            Out => '.',
            LBrac => '[',
            RBrac => ']',
        }
    }

    pub fn show_code(code: &[BFInst]) -> String {
        let mut s = String::new();

        for i in code {
            s.push(i.show());
        }

        s
    }
}

pub fn asm_to_bf(stack: &[StackInst]) -> Vec<BFInst> {
    let mut stack = Vec::from(stack);
    StackInst::expand(&mut stack);

    let mut bf = vec![];

    for inst in stack {
        use BFInst::*;
        use StackInst::*;
        match inst {
            PushB(b) => {
                bf.push(Right);
                bf.extend(repeat_n(Inc, b as _));
            }
            DiscardB => {
                bf.extend(BFInst::parse("[-]<"));
            }
            SwapB => {
                bf.extend(BFInst::parse(
                    "
                    [->+<]     // Copy 1
                    <[->+<]    // Shift 2 into 1
                    >>[-<<+>>] // Shift copy into 2
                    <          // Move to head
                    ",
                ));
            }
            CopyB => bf.extend(BFInst::parse("[->+<]")),
            AddB => bf.extend(BFInst::parse("[-<+>]<")),
            Alloc(n) => bf.extend(repeat_n(Right, n as _)),
            Dealloc(n) => bf.extend(repeat_n(Left, n as _)),
            LclReadB(n) => {
                let left = repeat_n(Left, n).collect::<Vec<_>>();
                let right = repeat_n(Right, n).collect::<Vec<_>>();
                bf.extend(&left);
                bf.extend(BFInst::parse("[-"));
                bf.extend(&right);
                bf.extend(BFInst::parse(">+<"));
                bf.extend(&left);
                bf.extend(BFInst::parse("]"));
            }
            LclStrB(n) => {
                let left = repeat_n(Left, n).collect::<Vec<_>>();
                let right = repeat_n(Right, n).collect::<Vec<_>>();
                bf.extend(&left);
                bf.extend(BFInst::parse("<[-]")); // Erase previous value
                bf.extend(&right);
                bf.extend(BFInst::parse(">[-")); // Enter move loop
                bf.extend(&left);
                bf.extend(BFInst::parse("<+")); // Shift 1 unit over
                bf.extend(&right);
                bf.extend(BFInst::parse(">]<")); // Exit loop and move stack head
            }
            Label(n) if n != 0 => {
                bf.extend(BFInst::parse("<[->+<]>")); // Move to, and then copy, label
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
            Goto => bf.extend(BFInst::parse(">]")),
            PrintChar => bf.push(Out),
            Label(0) | Nop | Debug(_) | Comment(_) => {}
            i => todo!("{:?}", i),
        }
    }

    bf
}
