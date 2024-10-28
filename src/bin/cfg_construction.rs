use bril::bril_syntax::Program;
use bril::cfg::CFG;
fn main() {
    // Filter out "nop" instructions for each function
    let mut prog = Program::stdin();

    let cfg = CFG::from_program(&mut prog);

    //cfg.analyze_loop();
    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
