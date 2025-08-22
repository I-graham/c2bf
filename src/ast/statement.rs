use super::*;

type Label = String;

pub enum Stmt {
    DefnStmt(Defn),
    ExprStmt(Expr),
    Labeled(Label, Box<Stmt>),
    Case(Expr, Box<Stmt>),
    Default(Box<Stmt>),
    SeqStmt(Vec<Stmt>),
    IfStmt(Expr, Box<Stmt>),
    IfElseStmt(Expr, Box<Stmt>, Box<Stmt>),
    SwitchStmt(Expr, Box<Stmt>),
    While(Expr, Box<Stmt>),
    DoWhile(Box<Stmt>, Expr),
    For(Expr, Option<Expr>, Expr, Box<Stmt>),
    Goto(Label),
    Continue,
    Break,
    Return(Option<Expr>),
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

            expr_stmt
                [e] -> ExprStmt(e);

            if_stmt
                [c, t] -> IfStmt(c, t);
                [c, t, e] -> IfElseStmt(c,t,e);

            switch_stmt
                [e, s] -> SwitchStmt(e, s);

            while_loop
                [e, s] -> While(e, s);

            do_loop
                [s, e] -> DoWhile(s, e);

            goto_stmt
                [s] -> Goto(s);

            continue_stmt
                [] -> Continue;

            break_stmt
                [] -> Break;

            return_stmt
                []  -> Return(None);
                [e] -> Return(Some(e));
        }
    }

    fn compile(&self, ctxt: &mut CompileContext, stream: &mut Vec<StackInst>) {
        use Stmt::*;
        match self {
            DefnStmt(d) => {
                let Defn::Vars(false, _, defs) = d else {
                    unreachable!();
                };

                for (decl, def) in defs {
                    let Some(def) = def else { continue };
                    let Some(v) = decl.get_name() else { continue };

                    def.compile(ctxt, stream);
                    ctxt.store(&v, stream);
                }
            }
            ExprStmt(expr) => {
                expr.compile(ctxt, stream);
                stream.push(StackInst::DiscardW);
            }
            SeqStmt(stmts) => {
                for stmt in stmts {
                    stmt.compile(ctxt, stream);
                }
            }

            _ => todo!(),
        }
    }
}

impl Stmt {
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
            Goto(_) | Continue | Break | Return(_) | ExprStmt(_) => vec![],
            SwitchStmt(_, stmt)
            | While(_, stmt)
            | DoWhile(stmt, _)
            | For(_, _, _, stmt)
            | IfStmt(_, stmt)
            | Default(stmt)
            | Case(_, stmt)
            | Labeled(_, stmt) => stmt.vars(),
            SeqStmt(stmts) => stmts.iter().flat_map(|s| s.vars()).collect(),
            IfElseStmt(_, s1, s2) => {
                let mut vs = s1.vars();
                vs.extend(s2.vars());
                vs
            }
        }
    }
}
