use super::*;

pub enum Expr {
    Const(u8),
    Var(String),
    BinOp(Op, Box<Expr>, Box<Expr>),
}

impl ASTNode for Expr {
    fn parse(pair: Pair<Rule>) -> Self {
        parser_rule! {
            pair p:

            IDENTIFIER -> Self::Var(p.into());
            CONSTANT -> Self::Const(p.parse::<u8>().unwrap());
            primary_expr [e] -> e;
            mul_expr [lhs, op, rhs] -> Self::BinOp(op, Box::new(lhs), Box::new(rhs))
        }
    }

    fn compile(&self, context: &CompileContext, stream: &mut Vec<StackInst>) {
        use StackInst::*;
        match self {
            Self::Const(v) => stream.push(Byte(*v)),
            Self::Var(_) => todo!(),
            Self::BinOp(op, lhs, rhs) => {
                let op = match op {
                    Op::Add => Add,
                    Op::Sub => Sub,
                    Op::Mul => Mul,
                    Op::Div => Div,
                };

                lhs.compile(context, stream);
                rhs.compile(context, stream);
                stream.push(op);
            }
        };
    }
}
