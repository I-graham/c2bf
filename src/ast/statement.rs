use super::*;

type Label = String;

pub enum Stmt {
    DefnStmt(Defn),
    ExprStmt(Option<Expr>),
    Labeled(Label, Box<Stmt>),
    Case(Expr, Box<Stmt>),
    Default(Box<Stmt>),
    SeqStmt(Vec<Stmt>),
    IfStmt(Expr, Box<Stmt>),
    IfElseStmt(Expr, Box<Stmt>, Box<Stmt>),
    SwitchStmt(Expr, Box<Stmt>),
    While(Expr, Box<Stmt>),
    DoWhile(Box<Stmt>, Expr),
    For(Box<Stmt>, Option<Expr>, Option<Expr>, Box<Stmt>),
    GotoStmt(Label),
    Continue,
    Break,
    Return(Option<Expr>),
    Print(Expr),
}

impl ASTNode for Stmt {
    fn parse(pair: Pair<Rule>) -> Self {
        use Stmt::*;
        parser_rule! {
            pair:

            stmt
            | selection_stmt
            | iteration_stmt
            | jump_stmt
                [s] -> s;

            declaration
                [d] -> DefnStmt(d);

            labeled_stmt
                [s] -> s;
                [l, s] -> Labeled(l, s);

            case_stmt
                [e, s] -> Case(e, s);

            default_stmt
                [s] -> Default(s);

            compound_stmt
                [.. ss,] -> SeqStmt(ss.map(Self::parse).collect());

            print_stmt
                [e] -> Print(e);

            expr_stmt
                [] -> ExprStmt(None);
                [e] -> ExprStmt(Some(e));

            if_stmt
                [c, t] -> IfStmt(c, t);
                [c, t, e] -> IfElseStmt(c,t,e);

            switch_stmt
                [e, s] -> SwitchStmt(e, s);

            while_loop
                [e, s] -> While(e, s);

            do_loop
                [s, e] -> DoWhile(s, e);

            for_loop
                [init, cond, body] -> {
                    let ExprStmt(cond) = cond else {
                        unreachable!()
                    };

                    For(init, cond, None, body)
                };
                [init, cond, end, body] -> {
                    let ExprStmt(cond) = cond else {
                        unreachable!()
                    };

                    For(init, cond, Some(end), body)
                };

            goto_stmt
                [s] -> GotoStmt(s);

            continue_stmt
                [] -> Continue;

            break_stmt
                [] -> Break;

            return_stmt
                []  -> Return(None);
                [e] -> Return(Some(e));
        }
    }

    fn compile(&self, ctxt: &mut CompileContext) {
        use StackInst::*;
        use Stmt::*;
        match self {
            DefnStmt(d) => {
                let Defn::Vars(false, _, defs) = d else {
                    unreachable!();
                };

                for (decl, def) in defs {
                    let Some(def) = def else { continue };
                    let Some(v) = decl.get_name() else { continue };

                    ctxt.compile(def);
                    ctxt.store(&v);
                }
            }
            ExprStmt(Some(expr)) => {
                ctxt.compile(expr);
                ctxt.emit(DiscardB);
            }
            ExprStmt(None) => {}
            SeqStmt(stmts) => {
                for stmt in stmts {
                    ctxt.compile(stmt);
                }
            }

            Print(expr) => {
                ctxt.compile(expr);
                ctxt.emit(PrintChar);
            }

            Return(e) => {
                if let Some(expr) = e {
                    ctxt.compile(expr);
                    ctxt.emit_stream(&[
                        Move(ctxt.local_offset),    // replace stack pointer
                        Dealloc(ctxt.local_offset), // Dealloc stack pointer
                        SwapB,
                        Goto,
                    ]);
                } else {
                    ctxt.emit_stream(&[Dealloc(ctxt.local_offset), Goto]);
                }
            }
            IfStmt(cond, body) => {
                let t_lbl = ctxt.label();
                let e_lbl = ctxt.label();

                ctxt.compile(cond);
                ctxt.emit_stream(&[Branch(t_lbl, e_lbl), Label(t_lbl)]);
                ctxt.compile(body);
                ctxt.emit_stream(&[PushB(e_lbl), Goto, Label(e_lbl)]);
            }
            IfElseStmt(cond, t_body, f_body) => {
                let t_lbl = ctxt.label();
                let f_lbl = ctxt.label();
                let e_lbl = ctxt.label();

                ctxt.compile(cond);
                ctxt.emit_stream(&[Branch(t_lbl, f_lbl), Label(t_lbl)]);
                ctxt.compile(t_body);
                ctxt.emit_stream(&[PushB(e_lbl), Goto, Label(f_lbl)]);
                ctxt.compile(f_body);
                ctxt.emit_stream(&[PushB(e_lbl), Goto, Label(e_lbl)]);
            }
            While(cond, body) => {
                let start = ctxt.label();
                let t_lbl = ctxt.label();
                let f_lbl = ctxt.label();

                ctxt.emit_stream(&[PushB(start), Goto, Label(start)]);
                ctxt.compile(cond);
                ctxt.emit_stream(&[Branch(t_lbl, f_lbl), Label(t_lbl)]);
                ctxt.compile(body);
                ctxt.emit_stream(&[PushB(start), Goto, Label(f_lbl)]);
            }
            For(init, cond, end, body) => {
                let c_lbl = ctxt.label();
                let b_lbl = ctxt.label();
                let leave = ctxt.label();

                ctxt.compile(init);
                ctxt.emit_stream(&[PushB(c_lbl), Goto, Label(c_lbl)]);
                match cond {
                    Some(cond) => ctxt.compile(cond),
                    None => ctxt.emit(PushB(1)),
                }
                ctxt.emit_stream(&[Branch(b_lbl, leave), Label(b_lbl)]);
                ctxt.compile(body);
                ctxt.compile(&ExprStmt(end.clone()));
                ctxt.emit_stream(&[PushB(c_lbl), Goto, Label(leave)]);
            }
            _ => todo!(),
        }
    }
}

impl Stmt {
    // TODO: Local scoping
    pub fn vars(&self) -> Vec<(DType, Option<Ident>)> {
        use Stmt::*;
        match self {
            DefnStmt(d) => {
                let Defn::Vars(_, base_ty, decls) = d else {
                    unreachable!()
                };

                decls
                    .iter()
                    .map(|(d, _)| (d.set_type(base_ty.clone()), d.get_name()))
                    .collect()
            }
            Print(_) | GotoStmt(_) | Continue | Break | Return(_) | ExprStmt(_) => vec![],
            SwitchStmt(_, stmt)
            | While(_, stmt)
            | DoWhile(stmt, _)
            | IfStmt(_, stmt)
            | Default(stmt)
            | Case(_, stmt)
            | Labeled(_, stmt) => stmt.vars(),
            SeqStmt(stmts) => stmts.iter().flat_map(|s| s.vars()).collect(),
            For(s1, _, _, s2) | IfElseStmt(_, s1, s2) => {
                let mut vs = s1.vars();
                vs.extend(s2.vars());
                vs
            }
        }
    }
}
