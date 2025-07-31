use super::*;

pub mod exec;

pub use exec::*;

pub type StackProgram = Vec<StackInst>;

#[derive(Default, Debug, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum StackInst {
    #[default]
    Nop,
    Byte(u8),
    Add,
    Sub,
    Mul,
    Div,
    Print,
}
