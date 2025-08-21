use super::*;

#[derive(Default, Debug, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum StackInst {
    // Misc. / Debug
    #[default]
    Nop,
    Comment(&'static str),

    // Preparation
    PushW(Word),

    // ALU
    Add,
    Sub,
    Mul,
    Div,

    // Memory
    StackAlloc,
    GlobalStore,
    GlobalRead,

    // Control Flow
    Label(Word),
    Goto,
    Exit,

    // IO
    ShowI32,
}
