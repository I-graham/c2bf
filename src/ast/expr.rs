use super::*;

#[derive(Clone, Debug)]
pub enum Expr {
    ConstW(u32),
    Var(Ident),
    BinOpExpr(Box<Expr>, Vec<(BinOp, Expr)>),
    Unary(MonOp, Box<Expr>),
    TypeSize(DType),
    Cast(DType, Box<Expr>),
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, BinOp, Box<Expr>),
    Seq(Vec<Expr>),
    InitList(Vec<Expr>),
    Indexed(Box<Expr>, Box<Expr>),
    FnCall(Box<Expr>, Vec<Expr>),
    Field(Box<Expr>, Ident),
    Arrow(Box<Expr>, Ident),
    Inc(Box<Expr>),
    Dec(Box<Expr>),
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
                [p, ..fixtures] -> {
                    let mut base = p;

                    for fixture in fixtures {
                        let boxed = Box::new(base);

                        base = match fixture.as_rule() {
                            index => Indexed(boxed, Self::parse(fixture).into()),
                            field => Field(boxed, fixture.as_str().into()),
                            arrow => Arrow(boxed, fixture.as_str().into()),
                            inc => Inc(boxed),
                            dec => Dec(boxed),
                            call => {
                                let args = fixture.into_inner().map(Expr::parse).collect();
                                FnCall(boxed, args)
                            }
                            _ => unreachable!(),
                        }
                    }

                    base
                };

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
                    BinOpExpr(acc, args)
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

    fn compile(&self, ctxt: &mut CompileContext) {
        use Expr::*;
        use StackInst::*;
        match self {
            ConstW(v) => ctxt.emit(PushW(*v)),
            Var(v) => ctxt.push_var(v),
            Unary(_op, _e) => todo!(),
            TypeSize(ty) => ctxt.emit(PushW(ty.size())),
            BinOpExpr(head, args) => {
                head.compile(ctxt);

                for (op, arg) in args {
                    let op = match op {
                        BinOp::Add => Add,
                        BinOp::Sub => Sub,
                        BinOp::Mul => Mul,
                        BinOp::Div => Div,
                        _ => todo!(),
                    };

                    arg.compile(ctxt);
                    ctxt.emit(op);
                }
            }
            FnCall(func, args) => {
                ctxt.call_fn(func, args);
            }
            Seq(seqs) => {
                let mut seqs = seqs.iter();
                seqs.next().unwrap().compile(ctxt);

                for seq in seqs {
                    ctxt.emit(DiscardW);
                    seq.compile(ctxt);
                }
            }

            e => todo!("Unsupported expr {:?}", e),
        };
    }
}

impl Expr {
    pub fn const_arithmetic_expr(&self) -> Option<u64> {
        use Expr::*;
        match self {
            ConstW(v) => Some(*v as u64),
            BinOpExpr(expr, items) => {
                let mut e_val = expr.const_arithmetic_expr()?;
                for (op, e) in items {
                    let operand = e.const_arithmetic_expr()?;
                    use BinOp::*;
                    e_val = match op {
                        Add => e_val + operand,
                        Sub => e_val - operand,
                        Mul => e_val * operand,
                        Div => e_val / operand,
                        _ => todo!(),
                    };
                }

                Some(e_val)
            }
            Unary(op, expr) => {
                let e_val = expr.const_arithmetic_expr()?;
                use MonOp::*;
                match op {
                    LogicalNot => Some(!e_val),
                    _ => todo!(),
                }
            }
            TypeSize(dtype) => Some(dtype.size() as u64),
            _ => None,
        }
    }
}
