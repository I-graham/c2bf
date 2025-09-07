use super::*;

#[derive(Clone, Debug)]
pub enum Expr {
    Const(usize),
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
            CONSTANT [e] -> e;
            decimal [] -> Const(s.parse::<usize>().unwrap());
            octal [] -> Const(usize::from_str_radix(s, 8).unwrap());
            hexadecimal [] -> Const(usize::from_str_radix(s, 16).unwrap());
            character [] -> Const(s.chars().nth(1).unwrap() as usize);
            string_literal [] -> todo!();

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
            Const(v) => ctxt.emit(Push(*v as Word)),
            Var(v) => ctxt.push_var(v),
            Unary(op, e) => {
                ctxt.compile(e);
                let op = match op {
                    MonOp::LogicalNot => LNot,
                    MonOp::BinaryNot => Not,
                    MonOp::Inc => todo!(),
                    MonOp::Dec => todo!(),
                    MonOp::SizeOf => todo!(),
                    MonOp::Deref => todo!(),
                    MonOp::Negate => todo!(),
                    MonOp::AddrOf => todo!(),
                };
                ctxt.emit(op);
            }
            TypeSize(ty) => ctxt.emit(Push(ty.size())),
            BinOpExpr(head, args) => {
                ctxt.compile(head);

                for (op, arg) in args {
                    let op = match op {
                        BinOp::Add => Add,
                        BinOp::Sub => Sub,
                        BinOp::Mul => Mul,
                        BinOp::Div => Div,
                        BinOp::Eq => Eq,
                        BinOp::Neq => Neq,
                        BinOp::Lt => Lt,
                        BinOp::LtEq => LtEq,
                        BinOp::Gr => Gr,
                        BinOp::GrEq => GrEq,
                        BinOp::LAnd => LAnd,
                        BinOp::LOr => LOr,
                        BinOp::LShift => LShift,
                        BinOp::RShift => RShift,
                        BinOp::And => And,
                        BinOp::Or => Or,
                        BinOp::Xor => Xor,
                        _ => todo!(),
                    };

                    ctxt.compile(arg);
                    ctxt.emit(op);
                }
            }
            FnCall(func, args) => {
                ctxt.call_fn(func, args);
            }
            Seq(seqs) => {
                let mut seqs = seqs.iter();
                ctxt.compile(seqs.next().unwrap());

                for seq in seqs {
                    ctxt.emit(Dealloc(1));
                    ctxt.compile(seq);
                }
            }

            Assign(var, BinOp::Set, val) => {
                let Expr::Var(v) = &**var else { todo!() };
                ctxt.compile(val);
                ctxt.emit(Copy);
                ctxt.store(v);
            }

            Inc(e) => {
                let Expr::Var(v) = &**e else { todo!() };
                ctxt.push_var(v);
                ctxt.emit_stream(&[Copy, Push(1), Add]);
                ctxt.store(v);
            }
            Dec(e) => {
                let Expr::Var(v) = &**e else { todo!() };
                ctxt.push_var(v);
                ctxt.emit_stream(&[Copy, Push(1), Sub]);
                ctxt.store(v);
            }

            e => todo!("Unsupported expr {:?}", e),
        };
    }
}

impl Expr {
    pub fn const_arithmetic_expr(&self) -> Option<u64> {
        use Expr::*;
        match self {
            Const(v) => Some(*v as u64),
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
                        Mod => e_val % operand,
                        LShift => e_val << operand,
                        RShift => e_val >> operand,
                        Xor => e_val ^ operand,
                        And => e_val & operand,
                        Or => e_val | operand,
                        _ => todo!(),
                    };
                }

                Some(e_val)
            }
            Unary(op, expr) => {
                let e_val = expr.const_arithmetic_expr()?;
                use MonOp::*;
                match op {
                    BinaryNot => Some(!e_val),
                    _ => todo!(),
                }
            }
            TypeSize(dtype) => Some(dtype.size() as u64),
            _ => None,
        }
    }
}
