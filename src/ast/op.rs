use super::*;

pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl ASTNode for Op {
    fn parse(pair: Pair<Rule>) -> Self {
        use Op::*;
        match pair.as_str() {
            "+" => Add,
            "-" => Sub,
            "*" => Mul,
            "/" => Div,

            s => unimplemented!("Unknown op `{}`", s),
        }
    }
}
