use super::*;

pub enum Expr {
    ConstW(u32),
    Var(Ident),
    Assoc(Box<Expr>, Vec<(Op, Expr)>),
    Unary(Op, Box<Expr>),
    TypeSize(DType),
    Cast(DType, Box<Expr>),
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, Op, Box<Expr>),
    Seq(Vec<Expr>),
    InitList(Vec<Expr>),
}

impl ASTNode for Expr {
    fn parse(pair: Pair<Rule>) -> Self {
        use Expr::*;
        let s = pair.as_str();
        parser_rule! {
            pair:

            IDENTIFIER [] -> Var(s.into());
            CONSTANT [] -> ConstW(s.parse::<u32>().unwrap());

            primary_expr
                [e] -> e;

            postfix_expr // TODO
                [e] -> e;

            const_expr
                [e] -> e;

            unary_expr
                [e] -> e;
                [op, e] -> Unary(op, e);

            type_size_expr
                [ty] -> TypeSize(ty);

            cast_expr
                [e] -> e;
                [t, c] -> Cast(t, c);

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
                    Assoc(acc, args)
                };

            conditional_expr
                [e] -> e;
                [c, t, f] -> Cond(c,t,f);

            assign_expr
                [e] -> e;
                [a, o, e] -> Assign(a, o, e);

            expr
                [.. es,] -> {
                    Seq(es.map(Self::parse).collect())
                };


            initializer
                [e] -> e;

            initializer_list
                [.. es,] -> InitList(es.map(Self::parse).collect());
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

            _ => todo!(),
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
            _ => None,
        }
    }
}
