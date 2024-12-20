use bril::alias_analysis::AliasAnalysis;
use bril::bril_syntax::Program;
use bril::cfg::CFG;
fn main() {
    // Filter out "nop" instructions for each function
    let mut prog = Program::stdin();

    let mut cfg = CFG::from_program(&mut prog);
    cfg.place_phi_functions_and_generate_ssa();

    let mut alias = AliasAnalysis::new(&cfg);
    cfg.dataflow(&mut alias);
    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
