use super::*;

type Label = String;

pub enum Stmt {
    Decl(Definition),
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

            labeled_stmt
                [s] -> s;
                [l, s] -> Labeled(l, s);

            case_stmt
                [e, s] -> Case(e, s);

            default_stmt
                [s] -> Default(s);

            compount_stmt
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

    fn compile(&self, context: &CompileContext, stream: &mut Vec<StackInst>) {
        match self {
            Self::ExprStmt(expr) => expr.compile(context, stream),

            _ => todo!(),
        }
    }
}
