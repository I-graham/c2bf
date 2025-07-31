use super::*;

pub enum ASTStatement {
    Expr(Expr),
}

impl ASTNode for ASTStatement {
    fn parse(pairs: Pair<Rule>) -> Self {
        todo!()
    }

    fn compile(&self, context: &CompileContext, stream: &mut Vec<StackInst>) {
        match self {
            Self::Expr(expr) => expr.compile(context, stream),
        }
    }
}

