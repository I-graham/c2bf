use super::*;

pub enum Stmt {
    ExprStmt(Expr),
    Labeled(String, Box<Stmt>),
    Case(Expr, Box<Stmt>),
    Default(Box<Stmt>),
    SeqStmt(Vec<Stmt>),
    IfStmt(Expr, Box<Stmt>),
    IfElseStmt(Expr, Box<Stmt>, Box<Stmt>),
    While(Expr, Box<Stmt>),
    DoWhile(Box<Stmt>, Expr),
    For(Expr, Option<Expr>, Expr, Box<Stmt>),
}

impl ASTNode for Stmt {
    fn parse(pair: Pair<Rule>) -> Self {
        use Stmt::*;
        parser_rule! {
            pair:

            stmt
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

            selection_stmt
                [e] -> e;

            if_stmt
                [c, t] -> IfStmt(c, t);
                [c, t, e] -> IfElseStmt(c,t,e);

            while_loop
                [e, s] -> While(e, s);

            do_loop
                [s, e] -> DoWhile(s, e);
        }
    }

    fn compile(&self, context: &CompileContext, stream: &mut Vec<StackInst>) {
        match self {
            Self::ExprStmt(expr) => expr.compile(context, stream),

            _ => todo!(),
        }
    }
}
