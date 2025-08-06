pub mod exec;

pub use exec::*;

pub type StackProgram = Vec<StackInst>;

#[derive(Default, Debug, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum StackInst {
    #[default]
    Nop,
    PushW(u32),
    Add,
    Sub,
    Mul,
    Div,
    Print,
}
