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

    dbg!(&stream);

    let transpilation = translate(&stream);
    println!("{}", show_bf(&transpilation, cfg!(feature = "debugbf")));

    let optimized = optimize(&transpilation);

    println!("\nExecution:\n");

    let profile = exec_fastbf(&optimized);

    if cfg!(feature = "profile") {
        let mut sorted = profile.into_iter().collect::<Vec<_>>();
        sorted.sort_unstable_by_key(|(_, t)| *t);

        println!("\n\nProfile:\n{:?}", sorted);
    }
}
