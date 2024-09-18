use bril::bril_syntax::{Instruction, InstructionOrLabel, Label, Program};

fn main() {
    // Filter out "nop" instructions for each function
    let mut prog = Program::stdin();

    for func in &mut prog.functions {
        func.instrs.retain(|instr| match instr {
            InstructionOrLabel::Instruction(a) => !a.is_nop(),
            InstructionOrLabel::Label(_) => false,
        });
    }

    // Serialize the modified program to stdout
    //
    prog.stdout();
}
