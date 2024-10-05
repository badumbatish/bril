use bril::bril_syntax::Program;
use bril::cfg::CFG;
fn main() {
    // Filter out "nop" instructions for each function
    let prog = Program::stdin();

    let mut cfg = CFG::from_program(prog);
    cfg.place_phi_functions();
    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
