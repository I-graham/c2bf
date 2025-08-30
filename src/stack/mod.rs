pub mod ctxt;
pub mod exec;
pub mod inst;

pub use ctxt::*;
pub use exec::*;
pub use inst::*;

pub use super::*;

pub type Word = u8;
pub const WORD_SIZE: usize = 4;
