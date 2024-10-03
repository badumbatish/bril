use bril::{bril_syntax::Program, cfg::CFG, dominance::DominanceDataFlow};

fn main() {
    let prog = Program::stdin();

    let cfg = CFG::from_program(prog);
    let mut dominance = DominanceDataFlow::new(&cfg);
    cfg.dataflow(&mut dominance);
    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
