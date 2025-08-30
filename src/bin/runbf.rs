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

    let CompileContext { stream, .. } = ctxt;

    println!("\nExecution:\n");

    let transpilation = asm_to_bf(&stream);

    exec_bf(&transpilation);
}
