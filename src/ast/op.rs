use super::*;

pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Not,
}

impl ASTNode for Op {
    fn parse(pair: Pair<Rule>) -> Self {
        use Op::*;
        match pair.as_str() {
            "+" => Add,
            "-" => Sub,
            "*" => Mul,
            "/" => Div,

            "!" => Not,

            s => unimplemented!("Unknown op `{}`", s),
        }
    }
}
