pub mod ast;
pub mod parser;
pub mod stack;

pub use ast::*;
pub use parser::*;
pub use stack::*;

use pest::Parser;

type Base = Program;

pub fn exec_file(filename: &str) {
    let file = &std::fs::read_to_string(filename).unwrap();

    let pair = CParser::parse(Rule::translation_unit, file)
        .unwrap()
        .next()
        .unwrap();

    let parsed = Base::parse(pair);

    let mut ctxt = CompileContext::default();

    parsed.compile(&mut ctxt);

    let CompileContext { stream, .. } = ctxt;

    dbg!(&stream);

    println!("\nExecution:\n");

    exec_stack_program(&stream);
}
