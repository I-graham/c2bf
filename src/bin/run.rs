use c2bf::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    exec_file(&args[1]);
}
