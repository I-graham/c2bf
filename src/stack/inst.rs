use super::*;

#[derive(Default, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum StackInst {
    // Misc. + Debug
    #[default]
    Nop,
    Comment(&'static str),
    Debug(&'static str),

    // Preparation
    PushW(Word),
    DiscardW,
    Move(usize), // Copy word into stack
    SwapW,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,

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
    StackAlloc,
    StackDealloc,
    GlobalStore,
    GlobalRead,
    LocalStore(usize), // Offset from top of stack
    LocalRead(usize),  // Offset from top of stack

    // Control Flow
    Label(Word),
    Branch(Word, Word), // (True label, False label)
    Goto,
    Exit,

    // IO
    PrintI32,
}

impl StackInst {
    // # of words of input + # of words of output (if constant)
    pub fn signature(self) -> (usize, Option<usize>) {
        use StackInst::*;
        match self {
            Comment(_) | Debug(_) | Nop => (0, Some(0)),
            PushW(_) => (0, Some(1)),
            DiscardW => (1, Some(0)),
            Move(_) => (1, Some(0)),

            SwapW => (2, Some(2)),
            LNot => (1, Some(1)),
            Add | Sub | Mul | Div | Eq | Neq | Lt | LtEq | Gr | GrEq | LAnd | LOr => (2, Some(1)),
            StackAlloc | StackDealloc => (1, None),
            GlobalStore => (2, Some(0)),
            GlobalRead => (1, Some(1)),
            LocalStore(_) => (1, Some(0)),
            LocalRead(_) => (0, Some(1)),
            Label(_) => (0, None),
            Branch(_, _) => (1, Some(0)),
            Goto => (1, Some(0)),
            Exit => (0, None),
            PrintI32 => (1, Some(0)),
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
            PushW(c) => write!(f, "PushW({})", c),
            DiscardW => write!(f, "DiscardW"),
            Move(d) => write!(f, "CopyDown({})", d),
            SwapW => write!(f, "SwapW"),
            Add => write!(f, "Add"),
            Sub => write!(f, "Sub"),
            Mul => write!(f, "Mul"),
            Div => write!(f, "Div"),
            StackAlloc => write!(f, "StackAlloc"),
            StackDealloc => write!(f, "StackDealloc"),
            GlobalStore => write!(f, "GlobalStore"),
            GlobalRead => write!(f, "GlobalRead"),
            LocalStore(d) => write!(f, "LocalStore({})", d),
            LocalRead(d) => write!(f, "LocalRead({})", d),
            Label(l) => write!(f, "Label({})", l),
            Branch(t, e) => write!(f, "Branch({}, {})", t, e),
            Goto => write!(f, "Goto"),
            Exit => write!(f, "Exit"),
            PrintI32 => write!(f, "PrintI32"),
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
