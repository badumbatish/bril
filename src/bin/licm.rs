use bril::bril_syntax::Program;
use bril::cfg::CFG;
fn main() {
    // Filter out "nop" instructions for each function
    let mut prog = Program::stdin();

    // Assumes that we piped from ssa form
    let mut cfg = CFG::from_program(&mut prog);
    cfg.place_phi_functions_and_generate_ssa();
    cfg.analyze_loop();
    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
