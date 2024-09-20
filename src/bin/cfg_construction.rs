use bril::bril_syntax::Program;
use bril::util::CFG;
fn main() {
    // Filter out "nop" instructions for each function
    let prog = Program::stdin();

    let cfg = CFG::<String>::from_program(prog);

    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
