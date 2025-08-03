use super::*;
use pest::iterators::Pair;

pub mod dtype;
pub mod expr;
pub mod op;
pub mod statement;

pub use dtype::*;
pub use expr::*;
pub use op::*;
pub use statement::*;

pub type Ident = &'static str;

pub trait ASTNode {
    fn parse(pair: Pair<Rule>) -> Self;

    fn compile(&self, _context: &CompileContext, _stream: &mut Vec<StackInst>) {
        unimplemented!()
    }
}

pub struct AST {
    stmts: Vec<ASTStatement>,
}

#[derive(Default)]
pub struct CompileContext {}

impl ASTNode for AST {
    fn parse(_: Pair<Rule>) -> Self {
        todo!()
    }

    fn compile(&self, context: &CompileContext, stream: &mut Vec<StackInst>) {
        for statement in &self.stmts {
            statement.compile(context, stream)
        }
    }
}

// Useful for trashing inputs
impl ASTNode for () {
    fn parse(_: Pair<Rule>) -> Self {}
}

#[macro_export]
macro_rules! parser_rule {
    ($pair:ident $($s:ident)?:
        $(
            $($rules:ident)|+ $([$(.. $v1:ident ,)? $($nonterm:ident $(: $ty:ty)?),* $(, .. $v2:ident)?] -> $out:expr);+ ;
        )*
    ) => {
        use Rule::*;
        $(
            let $s = $pair.as_str();
        )?
        let pairs = $pair.clone().into_inner().collect::<Vec<_>>();
        let rule = $pair.as_rule();
        match rule {
            $(
                $($rules)|+ => match &pairs[..] {
                    $([$($v1 @ .. ,)? $($nonterm),* $(, $v2 @ ..)?] => {
                        $(
                            let $v1 = $v1.into_iter().cloned();
                        )?

                        $(
                            let $nonterm = $nonterm.clone();
                            let $nonterm = ASTNode::parse($nonterm);
                            $(
                                let $nonterm : $ty = $nonterm;
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
