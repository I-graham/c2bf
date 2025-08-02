use super::*;

pub enum Expr {
    Const(u8),
    Var(String),
    AssocOp(Box<Expr>, Vec<(Op, Expr)>),
}

impl ASTNode for Expr {
    fn parse(pair: Pair<Rule>) -> Self {
        parser_rule! {
            pair p:

            IDENTIFIER [] -> Self::Var(p.into());
            CONSTANT [] -> Self::Const(p.parse::<u8>().unwrap());
            primary_expr
            | postfix_expr
            | cast_expr
            | unary_expr
                [e] -> e;
            add_expr
            | mul_expr
            | shift_expr
            | rel_expr
            | eq_expr
            | and_expr
            | xor_expr
            | ior_expr
            | land_expr
            | lor_expr
                [e] -> e;
                [acc, .. xs ] -> {
                    let mut args = vec![];
                    while let Some((op, arg)) = xs.next().zip(xs.next()) {
                        args.push((
                            ASTNode::parse(op),
                            ASTNode::parse(arg)
                        ));
                    }
                    Self::AssocOp(Box::new(acc), args)
                };
        }
    }

    fn compile(&self, context: &CompileContext, stream: &mut Vec<StackInst>) {
        use StackInst::*;
        match self {
            Self::Const(v) => stream.push(Byte(*v)),
            Self::Var(_) => todo!(),
            Self::AssocOp(head, args) => {
                head.compile(context, stream);

                for (op, arg) in args {
                    let op = match op {
                        Op::Add => Add,
                        Op::Sub => Sub,
                        Op::Mul => Mul,
                        Op::Div => Div,
                    };

                    arg.compile(context, stream);
                    stream.push(op);
                }
            }
        };
    }
}
