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
    And,
    Or,
    Xor,
    LogicalAnd,
    LogicalOr,
}

impl ASTNode for BinOp {
    fn parse(pair: Pair<Rule>) -> Self {
        use BinOp::*;
        match pair.as_str() {
            "+" => Add,
            "-" => Sub,
            "*" => Mul,
            "/" => Div,
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
