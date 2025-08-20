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
    let mut code = vec![];

    parsed.compile(&mut ctxt, &mut code);
    code.push(StackInst::ShowI32);

    exec_stack_program(&code);
}
