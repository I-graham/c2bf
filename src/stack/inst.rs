use super::*;

#[derive(Default, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum StackInst {
    // Misc. + Debug
    #[default]
    Nop,
    Comment(&'static str),
    Debug(usize),

    // Preparation
    PushW(Word),
    DiscardW,
    CopyDown(usize), // Copy word into stack
    SwapW,

    // ALU
    Add,
    Sub,
    Mul,
    Div,

    // Memory
    StackAlloc,
    StackDealloc,
    GlobalStore,
    GlobalRead,
    LocalStore(usize), // Offset from top of stack
    LocalRead(usize),  // Offset from top of stack

    // Control Flow
    Label(Word),
    Goto,
    Exit,

    // IO
    PrintI32,
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
            CopyDown(d) => write!(f, "CopyDown({})", d),
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
            Goto => write!(f, "Goto"),
            Exit => write!(f, "Exit"),
            PrintI32 => write!(f, "PrintI32"),
        }
    }
}
