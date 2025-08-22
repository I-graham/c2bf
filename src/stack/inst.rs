use super::*;

#[derive(Default, Debug, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum StackInst {
    // Misc. + Debug
    #[default]
    Nop,
    Comment(&'static str),

    // Preparation
    PushW(Word),
    DiscardW,

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
    LocalStore,
    LocalRead,

    // Control Flow
    Label(Word),
    Goto,
    Exit,

    // IO
    ShowI32,
}
