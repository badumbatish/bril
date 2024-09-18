use std::collections::HashMap;

use crate::bril_syntax::{Function, Instruction};

const BRANCH_INSTRUCTIONS: [&str; 4] = ["jmp", "br", "call", "ret"];

//#[derive(Debug)]
//struct Instruction {
//    op: String,
//    args: Vec<String>,
//}

#[derive(Debug)]
struct BasicBlock {
    entry_label: String,
    instrs: Vec<Instruction>,
    // predecessors: Vec<BasicBlock>, // TODO: needed for cyclic CFGs?
    successors: Vec<BasicBlock>,
}

impl BasicBlock {
    fn new(entry_label: String, instrs: Vec<Instruction>) -> Self {
        BasicBlock {
            entry_label,
            instrs,
            // predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }

    // fn add_predecessor(&mut self, pred: String) {
    //     self.predecessors.push(pred);
    // }
}

#[derive(Debug)]
struct CFG {
    function: Function,
    basic_blocks: Vec<BasicBlock>,
    entry_block: BasicBlock,
}

impl CFG {
    fn new(function: String, instructions: Vec<Instruction>) -> Self {
        let mut basic_blocks = Vec::new();

        let mut future_blocks = Vec::new();

        basic_blocks.push(BasicBlock::new("ENTRY".to_string(), Vec::new()));
        let entry_block = &mut basic_blocks[0];
        let mut curr_block = Some(entry_block);

        for inst in instructions {
            if let Some(op) = inst.op.as_str() {
                if BRANCH_INSTRUCTIONS.contains(&op) {
                    curr_block.instrs.push(inst);

                    // control flow aaaa
                    {
                        if op == "jmp" {
                            let succ = BasicBlock::new(inst.args[0].clone(), Vec::new());
                            curr_block.successors.push(succ);
                            basic_blocks.push(succ);
                            curr_block = &mut succ;
                        } else if op == "br" {
                            let left = BasicBlock::new(inst.args[1].clone(), Vec::new());
                            let right = BasicBlock::new(inst.args[2].clone(), Vec::new());
                            curr_block.successors.push(left);
                            curr_block.successors.push(right);
                            basic_blocks.push(left);
                            basic_blocks.push(right);
                            future_blocks.push((left, inst.args[0].clone()));
                            future_blocks.push((right, inst.args[0].clone()));
                        } else if op == "call" {
                            basic_blocks
                                .last_mut()
                                .unwrap()
                                .successors
                                .push(inst.args[0].clone());
                        } else if op == "ret" {
                            basic_blocks
                                .last_mut()
                                .unwrap()
                                .successors
                                .push("EXIT".to_string());
                        }
                    }
                    curr_instrs.clear();
                }
            } else if let Some(label) = inst.label.as_ref() {
                if !curr_instrs.is_empty() {
                    basic_blocks.push(BasicBlock::new(entry_label.clone(), curr_instrs.clone()));
                    curr_instrs.clear();
                }
                entry_label = label.clone();
            }
        }

        // Handle any remaining instructions
        if !curr_instrs.is_empty() {
            basic_blocks.push(BasicBlock::new(entry_label, curr_instrs));
        }

        CFG {
            function,
            basic_blocks,
            entry_block,
            exit_block,
        }
    }
}

impl std::fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.instrs)
    }
}

impl std::fmt::Display for CFG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bb in &self.basic_blocks {
            writeln!(f, "{}", bb)?;
        }
        Ok(())
    }
}

// Example usage
fn main() {
    let function_name = "example_function".to_string();
    let instructions = vec![
        Instruction {
            op: "label".to_string(),
            args: vec!["start".to_string()],
        },
        Instruction {
            op: "add".to_string(),
            args: vec!["a".to_string(), "b".to_string()],
        },
        Instruction {
            op: "jmp".to_string(),
            args: vec!["end".to_string()],
        },
        Instruction {
            op: "label".to_string(),
            args: vec!["end".to_string()],
        },
        Instruction {
            op: "ret".to_string(),
            args: vec![],
        },
    ];

    let cfg = CFG::new(function_name, instructions);
    println!("Control Flow Graph:\n{}", cfg);
}
