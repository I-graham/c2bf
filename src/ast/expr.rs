use super::*;

pub enum Expr {
    ConstW(u32),
    Var(String),
    Assoc(Box<Expr>, Vec<(Op, Expr)>),
    Unary(Op, Box<Expr>),
    TypeSize(DType),
}

impl ASTNode for Expr {
    fn parse(pair: Pair<Rule>) -> Self {
        parser_rule! {
            pair p:

            IDENTIFIER [] -> Self::Var(p.into());
            CONSTANT [] -> Self::ConstW(p.parse::<u32>().unwrap());
            primary_expr
            | postfix_expr
            | cast_expr
            | const_expr
                [e] -> e;
            unary_expr
                [e] -> e;
                [op, e] -> Self::Unary(op, Box::new(e));
            type_size_expr
                [ty] -> Self::TypeSize(ty);
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
                    Self::Assoc(Box::new(acc), args)
                };
            conditional_expr
                [e] -> e;
        }
    }

    fn compile(&self, context: &CompileContext, stream: &mut Vec<StackInst>) {
        use StackInst::*;
        match self {
            Self::ConstW(v) => stream.push(PushW(*v)),
            Self::Var(_) => todo!(),
            Self::Unary(_op, _e) => todo!(),
            Self::TypeSize(ty) => stream.push(PushW(ty.size())),
            Self::Assoc(head, args) => {
                head.compile(context, stream);

                for (op, arg) in args {
                    let op = match op {
                        Op::Add => Add,
                        Op::Sub => Sub,
                        Op::Mul => Mul,
                        Op::Div => Div,

                        _ => unreachable!(),
                    };

                    arg.compile(context, stream);
                    stream.push(op);
                }
            }
        };
    }
}

impl Expr {
    pub fn const_arithmetic_expr(&self) -> Option<u64> {
        use Expr::*;
        match self {
            ConstW(v) => Some(*v as u64),
            Assoc(expr, items) => {
                let mut e_val = expr.const_arithmetic_expr()?;
                for (op, e) in items {
                    let operand = e.const_arithmetic_expr()?;
                    e_val = match op {
                        Op::Add => e_val + operand,
                        Op::Sub => e_val - operand,
                        Op::Mul => e_val * operand,
                        Op::Div => e_val / operand,

                        _ => unreachable!(),
                    };
                }

                Some(e_val)
            }
            Unary(op, expr) => {
                let e_val = expr.const_arithmetic_expr()?;
                match op {
                    Op::Not => Some(!e_val),
                    _ => unreachable!(),
                }
            }
            TypeSize(dtype) => Some(dtype.size() as u64),
            Var(_) => None,
        }
    }
}
