use super::*;
use pest::iterators::Pair;

pub mod declarator;
pub mod definition;
pub mod dtype;
pub mod expr;
pub mod op;
pub mod program;
pub mod statement;

pub use declarator::*;
pub use definition::*;
pub use dtype::*;
pub use expr::*;
pub use op::*;
pub use program::*;
pub use statement::*;

pub type Ident = String;

pub trait ASTNode {
    fn parse(pair: Pair<Rule>) -> Self;

    fn compile(&self, _context: &CompileContext, _stream: &mut Vec<StackInst>) {
        unimplemented!()
    }
}

#[derive(Default)]
pub struct CompileContext {}

// Useful for trashing inputs
impl ASTNode for () {
    fn parse(_: Pair<Rule>) -> Self {}
}

// Useful for reading strings
impl ASTNode for String {
    fn parse(pair: Pair<Rule>) -> Self {
        pair.as_str().into()
    }
}

impl<T: ASTNode> ASTNode for Box<T> {
    fn parse(pair: Pair<Rule>) -> Self {
        T::parse(pair).into()
    }

    fn compile(&self, context: &CompileContext, stream: &mut Vec<StackInst>) {
        (**self).compile(context, stream);
    }
}

#[macro_export]
macro_rules! parser_rule {
    ($pair:ident :
        $(
            $($rules:ident)|+ $([$(.. $v1:ident ,)? $($nonterm:ident $(: $ty:ty)?),* $(, .. $v2:ident)?] -> $out:expr);+ ;
        )*
    ) => {
        use Rule::*;
        let pairs = $pair.clone().into_inner().collect::<Vec<_>>();
        let rule = $pair.as_rule();
        #[allow(warnings)]
        #[allow(clippy::all)]
        match rule {
            $(
                $($rules)|+ => match &pairs[..] {
                    $([$($v1 @ .. ,)? $($nonterm),* $(, $v2 @ ..)?] => {
                        $(
                            let mut $v1 = $v1.into_iter().cloned();
                        )?

                        $(
                            let $nonterm = $nonterm.clone();
                            let mut $nonterm = ASTNode::parse($nonterm);
                            $(
                                let mut $nonterm : $ty = $nonterm;
                            )?
                        )*

                        $(
                            let mut $v2 = $v2.into_iter().cloned();
                        )?

                        $out
                    })+

                    _ => unimplemented!("Unable to match `{}`", $pair)
                }
            )*

            r => unimplemented!("Rule `{:?}` not matched.", r)
        }
    }
}
