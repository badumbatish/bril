use bril::bril_syntax::Program;

fn main() {
    // Filter out "nop" instructions for each function
    let mut prog = Program::stdin();

    for func in &mut prog.functions {
        func.instrs.retain(|instr| !instr.is_nop());
    }

    // Serialize the modified program to stdout
    //
    prog.stdout();
}
