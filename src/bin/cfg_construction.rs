use bril::bril_syntax::Program;
use bril::util::CFG;
fn main() {
    // Filter out "nop" instructions for each function
    let prog = Program::stdin();

    let cfg = CFG::from_program(&prog);
    println!("{}", cfg.to_dot_string());
    // cfg.print_hm();
}
