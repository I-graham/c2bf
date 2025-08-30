use c2bf::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let file = &std::fs::read_to_string(&args[1]).unwrap();

    let pair = CParser::parse(Rule::translation_unit, file)
        .unwrap()
        .next()
        .unwrap();

    let parsed = Program::parse(pair);

    let mut ctxt = CompileContext::default();

    parsed.compile(&mut ctxt);

    let CompileContext { mut stream, .. } = ctxt;

    StackInst::expand(&mut stream);

    dbg!(&stream);

    println!("\nExecution:\n");

    exec_stack_program(&stream);
}
