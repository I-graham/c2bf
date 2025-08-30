use c2bf::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let bf_code = &std::fs::read_to_string(&args[1]).unwrap();
    let code = BFInst::parse(bf_code);

    BFInst::show_code(&code, false);

    exec_bf(&code);
}
