use bril::{bril_syntax::Program, cfg::CFG, dominance::DominanceDataFlow};

fn main() {
    let mut prog = Program::stdin();

    let cfg = CFG::from_program(&mut prog);
    let _dominance = DominanceDataFlow::new(&cfg);

    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
