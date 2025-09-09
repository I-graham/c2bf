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
    Assign(Box<Expr>, AssignOp, Box<Expr>),
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
                            index => {
                                let id = fixture.into_inner().next().unwrap();
                                Indexed(boxed, Self::parse(id).into())
                            }
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

            addr_of
                [e] -> Unary(MonOp::AddrOf, e);

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
            Var(v) => {
                if let DType::Array(_, _) = ctxt.vty(v) {
                    ctxt.push_addr(v);
                    return;
                }
                ctxt.push_var(v);
            }
            Unary(MonOp::LogicalNot, e) => {
                ctxt.compile(e);
                ctxt.emit(LNot);
            }
            Unary(MonOp::BinaryNot, e) => {
                ctxt.compile(e);
                ctxt.emit(Not);
            }
            Unary(MonOp::Negate, e) => {
                ctxt.compile(e);
                ctxt.emit(Negate);
            }
            Unary(MonOp::Inc, e) => {
                let Expr::Var(v) = &**e else { todo!() };
                ctxt.push_var(v);
                ctxt.emit_stream(&[Push(1), Add, Copy]);
                ctxt.store(v);
            }
            Unary(MonOp::Dec, e) => {
                let Expr::Var(v) = &**e else { todo!() };
                ctxt.push_var(v);
                ctxt.emit_stream(&[Push(1), Sub, Copy]);
                ctxt.store(v);
            }
            Unary(MonOp::AddrOf, e) => {
                e.compile_addr(ctxt);
            }
            Unary(MonOp::Deref, e) => {
                ctxt.compile(e);
                let height = ctxt.stack_height.unwrap();
                ctxt.emit_stream(&[
                    LclRead(height - 1),
                    Push(height as Word),
                    Add,
                    Swap,
                    Sub,
                    StkRead,
                ]);
            }
            Cond(c, t, f) => {
                let height = ctxt.stack_height.unwrap();
                let t_lbl = ctxt.label();
                let f_lbl = ctxt.label();
                let leave = ctxt.label();
                ctxt.compile(c);
                ctxt.emit_stream(&[Branch(t_lbl, f_lbl), Label(t_lbl)]);
                ctxt.stack_height = Some(height);
                ctxt.compile(t);
                ctxt.emit_stream(&[Push(leave), Goto, Label(f_lbl)]);
                ctxt.stack_height = Some(height);
                ctxt.compile(f);
                ctxt.emit_stream(&[Push(leave), Goto, Label(leave)]);
                ctxt.stack_height = Some(height + 1);
            }
            TypeSize(ty) => ctxt.emit(Push(ty.size())),
            BinOpExpr(head, args) => {
                ctxt.compile(head);

                // Short-circuiting And
                if let Some((BinOp::LAnd, _)) = args.first() {
                    let height = ctxt.stack_height;
                    let last = ctxt.label();
                    let fail = ctxt.label();

                    for (_, arg) in args {
                        ctxt.stack_height = height;
                        let cont = ctxt.label();
                        ctxt.emit_stream(&[Branch(cont, fail), Label(cont)]);
                        ctxt.compile(arg);
                    }

                    ctxt.emit_stream(&[
                        Push(last),
                        Goto,
                        Label(fail),
                        Push(0),
                        Push(last),
                        Goto,
                        Label(last),
                    ]);
                    ctxt.stack_height = height;

                    return;
                }

                // Short-circuiting Or
                if let Some((BinOp::LOr, _)) = args.first() {
                    let height = ctxt.stack_height;
                    let succ = ctxt.label();
                    let last = ctxt.label();

                    for (_, arg) in args {
                        let cont = ctxt.label();
                        ctxt.stack_height = height;
                        ctxt.emit_stream(&[Branch(succ, cont), Label(cont)]);
                        ctxt.compile(arg);
                    }

                    ctxt.emit_stream(&[
                        Push(last),
                        Goto,
                        Label(succ),
                        Push(1),
                        Push(last),
                        Goto,
                        Label(last),
                    ]);
                    ctxt.stack_height = height;

                    return;
                }

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
                        BinOp::LShift => LShift,
                        BinOp::RShift => RShift,
                        BinOp::And => And,
                        BinOp::Or => Or,
                        BinOp::Xor => Xor,
                        BinOp::Mod => Mod,
                        _ => unreachable!(),
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

            Assign(var, AssignOp::Assign, val) => {
                ctxt.compile(val);
                ctxt.emit(Copy);
                if let Expr::Var(v) = &**var {
                    ctxt.store(v);
                } else {
                    var.compile_addr(ctxt);
                    let height = ctxt.stack_height.unwrap();
                    ctxt.emit_stream(&[
                        LclRead(height - 1),
                        Push(height as Word - 2),
                        Add,
                        Swap,
                        Sub,
                        StkStr,
                    ]);
                }
            }

            Assign(var, op, val) => {
                let op = match op {
                    AssignOp::MulAssign => Mul,
                    AssignOp::DivAssign => Div,
                    AssignOp::PlusAssign => Add,
                    AssignOp::SubAssign => Sub,
                    AssignOp::LShiftAssign => LShift,
                    AssignOp::RShiftAssign => RShift,
                    AssignOp::AndAssign => And,
                    AssignOp::OrAssign => Or,
                    AssignOp::XorAssign => Xor,
                    _ => unreachable!(),
                };

                var.compile_addr(ctxt);
                ctxt.emit(Copy);
                let height = ctxt.stack_height.unwrap();
                ctxt.emit_stream(&[
                    LclRead(height - 1),
                    Push(height as Word - 1),
                    Add,
                    Swap,
                    Sub,
                    StkRead,
                ]);

                ctxt.compile(val);
                ctxt.emit_stream(&[op, Swap, LclRead(1), Swap]);

                let height = ctxt.stack_height.unwrap();
                ctxt.emit_stream(&[
                    LclRead(height - 1),
                    Push(height as Word - 2),
                    Add,
                    Swap,
                    Sub,
                    StkStr,
                ]);
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

            Indexed(_, _) => {
                self.compile_addr(ctxt);
                let height = ctxt.stack_height.unwrap();
                ctxt.emit_stream(&[
                    LclRead(height - 1),
                    Push(height as Word - 1),
                    Add,
                    Swap,
                    Sub,
                    StkRead,
                ]);
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

    pub fn compile_addr(&self, ctxt: &mut CompileContext) {
        use Expr::*;
        use StackInst::*;
        match self {
            Var(v) => ctxt.push_addr(v),
            Unary(MonOp::Deref, e) => ctxt.compile(e),
            Seq(es) => {
                let n = es.len();
                for e in &es[0..n - 1] {
                    ctxt.compile(e);
                    ctxt.emit(Dealloc(1));
                }
                es[n - 1].compile_addr(ctxt);
            }
            Indexed(arr, id) => {
                ctxt.compile(arr);
                ctxt.compile(id);
                // Will not work for multidimensional arrays
                ctxt.emit(Add);
            }
            _ => unreachable!("{:?}", self),
        }
    }
}
