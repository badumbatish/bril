use bril::bril_syntax::{Instruction, InstructionOrLabel, Label, Program};
use bril::util::CFG;
fn main() {
    // Filter out "nop" instructions for each function
    let prog = Program::stdin();

    let hm = CFG::hm_from_program(&prog);
    CFG::print_hm(&hm);
}
