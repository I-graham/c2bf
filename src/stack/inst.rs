use super::*;

#[derive(Default, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum StackInst {
    // Misc. + Debug
    #[default]
    Nop,
    Comment(&'static str),
    Debug(&'static str),

    // Stack Manipulation
    PushB(Word),
    DiscardB,
    Move(usize), // Copy word into stack
    SwapB,
    CopyB,

    // Arithmetic
    AddB,
    SubB,
    MulB,
    DivB,

    // Comparison
    Eq,
    Neq,
    Lt,
    LtEq,
    Gr,
    GrEq,

    // Logical ops
    LNot,
    LAnd,
    LOr,

    // Memory
    Alloc(usize), // No runtime allocations yet
    Dealloc(usize),
    GblStrB,
    GblReadB,
    LclStrB(usize),  // Offset from top of stack
    LclReadB(usize), // Offset from top of stack

    // Control Flow
    Label(Word),
    Branch(Word, Word), // (True label, False label)
    Goto,
    Exit,

    // IO
    PrintChar,
}

impl StackInst {
    pub fn expand(stream: &mut Vec<Self>) {
        use StackInst::*;
        let mut out = vec![];

        stream.reverse();

        while let Some(inst) = stream.pop() {
            let mut expansion = match inst {
                Move(d) => [CopyB, LclReadB(d + 1)],
                Exit => [PushB(0), Goto],
                // All comparisons are in terms of GrEq
                LtEq => [SwapB, GrEq],
                Lt => [GrEq, LNot],
                Gr => [LtEq, LNot],
                _ => {
                    out.push(inst);
                    continue;
                }
            };
            expansion.reverse();
            stream.extend(expansion);
        }

        *stream = out;
    }

    // # of words of input + # of words of output (if constant)
    pub fn signature(self) -> (usize, Option<usize>) {
        use StackInst::*;
        match self {
            Comment(_) | Debug(_) | Nop => (0, Some(0)),
            PushB(_) => (0, Some(1)),
            DiscardB => (1, Some(0)),
            Move(_) => (1, Some(0)),
            CopyB => (1, Some(2)),

            SwapB => (2, Some(2)),
            LNot => (1, Some(1)),
            AddB | SubB | MulB | DivB | Eq | Neq | Lt | LtEq | Gr | GrEq | LAnd | LOr => {
                (2, Some(1))
            }
            Alloc(_) => (0, None),
            Dealloc(_) => (0, None),
            GblStrB => (2, Some(0)),
            GblReadB => (1, Some(1)),
            LclStrB(_) => (1, Some(0)),
            LclReadB(_) => (0, Some(1)),
            Label(_) => (0, None),
            Branch(_, _) => (1, Some(0)),
            Goto => (1, Some(0)),
            Exit => (0, None),
            PrintChar => (1, Some(0)),
        }
    }
}

impl std::fmt::Debug for StackInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StackInst::*;
        match self {
            Nop => write!(f, "Nop"),
            Debug(l) => write!(f, "Debug({})", l),
            Comment(c) => write!(f, "/* {} */", c),
            PushB(c) => write!(f, "PushB({})", c),
            DiscardB => write!(f, "DiscardB"),
            Move(d) => write!(f, "Move({})", d),
            SwapB => write!(f, "SwapB"),
            CopyB => write!(f, "CopyB"),
            AddB => write!(f, "AddB"),
            SubB => write!(f, "SubB"),
            MulB => write!(f, "MulB"),
            DivB => write!(f, "DivB"),
            Alloc(n) => write!(f, "Alloc({})", n),
            Dealloc(n) => write!(f, "Dealloc({})", n),
            GblStrB => write!(f, "GblStrB"),
            GblReadB => write!(f, "GblReadB"),
            LclStrB(d) => write!(f, "LclStrB({})", d),
            LclReadB(d) => write!(f, "LclReadB({})", d),
            Label(l) => write!(f, "Label({})", l),
            Branch(t, e) => write!(f, "Branch({}, {})", t, e),
            Goto => write!(f, "Goto"),
            Exit => write!(f, "Exit"),
            PrintChar => write!(f, "PrintChar"),
            Eq => write!(f, "Eq"),
            Neq => write!(f, "Neq"),
            Lt => write!(f, "Lt"),
            LtEq => write!(f, "LtEq"),
            Gr => write!(f, "Gr"),
            GrEq => write!(f, "GrEq"),
            LNot => write!(f, "LNot"),
            LAnd => write!(f, "LAnd"),
            LOr => write!(f, "LOr"),
        }
    }
}
