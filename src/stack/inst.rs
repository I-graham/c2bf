use super::*;

#[derive(Default, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum StackInst {
    // Misc. + Debug
    #[default]
    Nop,
    Comment(&'static str),
    Debug(&'static str),

    // Stack Manipulation
    Push(Word),
    Move(usize), // Copy word into stack
    Swap,
    Copy,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Negate,

    // Bitwise Ops
    LShift,
    RShift,
    And,
    Or,
    Xor,
    Not,

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
    GblStr,
    GblRead,
    LclStr(usize),  // Offset from top of stack
    LclRead(usize), // Offset from top of stack
    StkRead,
    StkStr,

    // Control Flow
    Label(Word),
    Branch(Word, Word), // (True label, False label)
    Goto,
    Exit,

    // IO
    PutChar,
}

impl StackInst {
    pub fn expand(stream: &mut Vec<Self>) {
        use StackInst::*;
        let mut out = vec![];

        stream.reverse();

        while let Some(inst) = stream.pop() {
            let expansion: &[_] = match inst {
                Move(d) => &[Copy, LclStr(d + 1)],
                Exit => &[Push(0), Goto],
                Eq => &[Neq, LNot],
                // All comparisons are in terms of GrEq
                LtEq => &[Swap, GrEq],
                Lt => &[GrEq, LNot],
                Gr => &[LtEq, LNot],
                _ => {
                    out.push(inst);
                    continue;
                }
            };
            for &inst in expansion.iter().rev() {
                stream.push(inst);
            }
        }

        *stream = out;
    }

    // # of words of input + # of words of output (if constant)
    pub fn signature(self) -> (usize, Option<usize>) {
        use StackInst::*;
        match self {
            Comment(_) | Debug(_) | Nop => (0, Some(0)),
            Push(_) => (0, Some(1)),
            Move(_) => (1, Some(0)),
            Copy => (1, Some(2)),
            Swap => (2, Some(2)),
            LNot | Not => (1, Some(1)),
            Add | Sub | Mul | Div | Eq | Neq | Lt | LtEq | Gr | GrEq | LAnd | LOr | LShift
            | RShift | And | Or | Xor | Mod => (2, Some(1)),
            Alloc(n) => (0, Some(n)),
            Dealloc(n) => (n, Some(0)),
            GblStr | StkStr => (2, Some(0)),
            GblRead | StkRead => (1, Some(1)),
            Negate => (1, Some(1)),
            LclStr(_) => (1, Some(0)),
            LclRead(_) => (0, Some(1)),
            Label(_) => (0, None),
            Branch(_, _) => (1, Some(0)),
            Goto => (1, Some(0)),
            Exit => (0, None),
            PutChar => (1, Some(0)),
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
            Push(c) => write!(f, "Push({})", c),
            Move(d) => write!(f, "Move({})", d),
            Swap => write!(f, "Swap"),
            Copy => write!(f, "Copy"),
            Add => write!(f, "Add"),
            Sub => write!(f, "Sub"),
            Mul => write!(f, "Mul"),
            Div => write!(f, "Div"),
            Mod => write!(f, "Mod"),
            Negate => write!(f, "Negate"),
            LShift => write!(f, "LShift"),
            RShift => write!(f, "RShift"),
            Alloc(n) => write!(f, "Alloc({})", n),
            Dealloc(n) => write!(f, "Dealloc({})", n),
            GblStr => write!(f, "GblStr"),
            GblRead => write!(f, "GblRead"),
            StkStr => write!(f, "StkStr"),
            StkRead => write!(f, "StkRead"),
            LclStr(d) => write!(f, "LclStr({})", d),
            LclRead(d) => write!(f, "LclRead({})", d),
            Label(l) => write!(f, "Label({})", l),
            Branch(t, e) => write!(f, "Branch({}, {})", t, e),
            Goto => write!(f, "Goto"),
            Exit => write!(f, "Exit"),
            PutChar => write!(f, "PrintChar"),
            Eq => write!(f, "Eq"),
            Neq => write!(f, "Neq"),
            Lt => write!(f, "Lt"),
            LtEq => write!(f, "LtEq"),
            Gr => write!(f, "Gr"),
            GrEq => write!(f, "GrEq"),
            LNot => write!(f, "LNot"),
            LAnd => write!(f, "LAnd"),
            LOr => write!(f, "LOr"),
            And => write!(f, "And"),
            Or => write!(f, "Or"),
            Xor => write!(f, "Xor"),
            Not => write!(f, "Not"),
        }
    }
}
