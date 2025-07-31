use super::*;
use pest::iterators::Pair;

pub mod expr;
pub mod op;
pub mod statement;

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

#[macro_export]
macro_rules! parser_rule {
    ($pair:ident $($s:ident)?:
        $(
            $rule:ident $([$($nonterm:ident),*])? -> $out:expr
        );*
    ) => {
        use Rule::*;
        $(
            let $s = $pair.as_str();
        )?
        match $pair.as_rule() {
            $($rule => {
                #[allow(unused)]
                let mut pairs = $pair.into_inner();
                $($(
                    let next = pairs.next().unwrap();
                    let $nonterm = ASTNode::parse(next);
                )*)?
                $out
            })*
            _ => unimplemented!()
        }
    }
}
