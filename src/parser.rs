use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "c.pest"]
pub struct CParser;
