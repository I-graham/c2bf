use super::*;

#[derive(Default, Debug, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum StackInst {
    #[default]
    // Preparation
    Nop,
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

    // IO
    ShowI32,
}
