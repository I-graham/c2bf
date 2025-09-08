use super::*;

#[derive(Clone, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LShift,
    RShift,
    Eq,
    Neq,
    Lt,
    LtEq,
    Gr,
    GrEq,
    LAnd,
    LOr,
    And,
    Or,
    Xor,
}

impl ASTNode for BinOp {
    fn parse(pair: Pair<Rule>) -> Self {
        use BinOp::*;
        match pair.as_str() {
            "+" => Add,
            "-" => Sub,
            "*" => Mul,
            "/" => Div,
            "%" => Mod,
            "==" => Eq,
            "!=" => Neq,
            "<" => Lt,
            "<=" => LtEq,
            ">" => Gr,
            ">=" => GrEq,
            "&&" => LAnd,
            "||" => LOr,
            "<<" => LShift,
            ">>" => RShift,
            "&" => And,
            "|" => Or,
            "^" => Xor,
            s => unimplemented!("Unknown op `{}`", s),
        }
    }
}

#[derive(Clone, Debug)]
pub enum MonOp {
    LogicalNot,
    Inc,
    Dec,
    SizeOf,
    Deref,
    Negate,
    BinaryNot,
    AddrOf,
}

impl ASTNode for MonOp {
    fn parse(pair: Pair<Rule>) -> Self {
        use MonOp::*;
        match pair.as_str() {
            "!" => LogicalNot,
            "++" => Inc,
            "--" => Dec,
            "sizeof" => SizeOf,
            "*" => Deref,
            "-" => Negate,
            "~" => BinaryNot,
            "&" => AddrOf,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AssignOp {
    Assign,
    MulAssign,
    DivAssign,
    ModAssign,
    PlusAssign,
    SubAssign,
    LShiftAssign,
    RShiftAssign,
    AndAssign,
    OrAssign,
    XorAssign,
}

impl ASTNode for AssignOp {
    fn parse(pair: Pair<Rule>) -> Self {
        use AssignOp::*;
        match pair.as_str() {
            "=" => Assign,
            "*=" => MulAssign,
            "/=" => DivAssign,
            "%=" => ModAssign,
            "+=" => PlusAssign,
            "-=" => SubAssign,
            "<<=" => LShiftAssign,
            ">>=" => RShiftAssign,
            "&=" => AndAssign,
            "|=" => OrAssign,
            "^=" => XorAssign,
            _ => unreachable!(),
        }
    }
}
