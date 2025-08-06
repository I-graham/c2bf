pub mod ast;
pub mod parser;
pub mod stack_machine;

pub use ast::*;
pub use parser::*;
pub use stack_machine::*;

use pest::Parser;

type Base = Expr;

pub fn exec_file(filename: &str) {
    let file = &std::fs::read_to_string(filename).unwrap();

    let pair = CParser::parse(Rule::type_size_expr, file)
        .unwrap()
        .next()
        .unwrap();

    let parsed = Base::parse(pair);

    let ctxt = CompileContext::default();
    let mut code = vec![];

    parsed.compile(&ctxt, &mut code);
    code.push(StackInst::Print);

    exec_stack_program(&code);
}
