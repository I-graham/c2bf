use super::*;

pub enum DType {
    Int,
    Float,
    Double,
    Pointer(Box<Self>),
}

impl ASTNode for DType {
    fn parse(pair: Pair<Rule>) -> Self {
        parser_rule! {
            pair:

            type_name
                [_q:()] -> Self::Int;
                [_q:(), t] -> t;

            abstract_declarator
                [d] -> d;
        }
    }
}
