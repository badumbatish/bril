use bril::bril_syntax::{InstructionOrLabel, Program};

fn main() {
    // Filter out "nop" instructions for each function
    let mut prog = Program::stdin();

    for func in &mut prog.functions {
        func.instrs.iter_mut().for_each(|instr| match instr {
            InstructionOrLabel::Instruction(p) => {
                if p.is_phi() {
                    if let Some(args) = &p.args {
                        if args.len() == 1 {
                            p.op = "id".to_string();
                            p.labels = None;
                        }
                    }
                }
            }
            InstructionOrLabel::Label(_) => (),
        });
    }

    // Serialize the modified program to stdout
    //
    prog.stdout();
}
