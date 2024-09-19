use core::panic;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::bril_syntax::{Function, InstructionOrLabel, Program};

#[derive(Hash, Debug, Eq, PartialEq, Clone)]
pub enum Leader {
    FunctionName(String),
    InstructionOrLabel(InstructionOrLabel),
}

#[derive(Debug)]
pub struct BasicBlock {
    leader: Leader,
    instrs: Vec<InstructionOrLabel>,
    predecessors: Vec<RefCell<BasicBlock>>,
    successors: Vec<RefCell<BasicBlock>>,
}
impl std::fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "----Basic Block----
----Leader: {:?}
----Instructions: \n",
            self.leader,
        )
        .unwrap();
        for instr in self.instrs.iter() {
            writeln!(f, "{:?}", instr).unwrap();
        }
        writeln!(f, "\n")
    }
}
impl BasicBlock {
    pub fn default() -> BasicBlock {
        Self {
            leader: Leader::FunctionName(String::default()),
            instrs: Vec::new(),
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }
    pub fn simple_basic_blocks_vec_from_function(f: &Function) -> Vec<Rc<RefCell<BasicBlock>>> {
        let mut result = Vec::new();
        let mut i = 0;
        // let mut last_instruction_before_construction = 0;
        while i < f.instrs.len() {
            match &f.instrs[i] {
                // this match only happens if instruction is at start of function or after a branch
                // without label
                InstructionOrLabel::Instruction(_) => {
                    let mut bb = BasicBlock::default();
                    if result.is_empty() {
                        bb.leader = Leader::FunctionName(f.name.clone());
                    } else {
                        panic!("I don't know how to handle this case of instruction happenning after a branch without label");
                    }

                    bb.instrs.push(f.instrs[i].clone());
                    i += 1;
                    loop {
                        if i >= f.instrs.len() {
                            break;
                        }
                        match &f.instrs[i] {
                            InstructionOrLabel::Instruction(instr) => {
                                bb.instrs
                                    .push(InstructionOrLabel::Instruction(instr.clone()));
                                if instr.is_nonlinear() {
                                    i += 1;
                                    break;
                                }
                            }
                            InstructionOrLabel::Label(_) => panic!("Cannot handle labels rn"),
                        }
                        i += 1;
                    }

                    result.push(Rc::new(RefCell::new(bb)));
                }
                InstructionOrLabel::Label(lb) => {
                    let mut bb = BasicBlock::default();
                    bb.leader = Leader::InstructionOrLabel(InstructionOrLabel::Label(lb.clone()));
                    bb.instrs.push(f.instrs[i].clone());

                    i += 1;
                    loop {
                        match &f.instrs[i] {
                            InstructionOrLabel::Instruction(_) => {}
                            InstructionOrLabel::Label(_) => {}
                        }
                    }
                    continue;
                }
            }
            i += 1;
        }
        result
    }
}
#[derive(Debug)]
pub struct CFG {
    entry: BasicBlock,
}
// main:
// @main
impl CFG {
    pub fn hm_from_function(f: Function) -> HashMap<Leader, Rc<RefCell<BasicBlock>>> {
        // O(n)
        let mut hm = HashMap::<Leader, Rc<RefCell<BasicBlock>>>::new();

        let simple_basic_blocks_vec_from_function =
            BasicBlock::simple_basic_blocks_vec_from_function(&f);
        for bb in simple_basic_blocks_vec_from_function {
            let bb_clone = bb.clone();
            hm.insert(bb_clone.borrow().leader.clone(), bb.clone());
        }

        hm
        // O(n)
    }
    pub fn hm_from_program(p: &Program) -> HashMap<Leader, Rc<RefCell<BasicBlock>>> {
        let mut hm = HashMap::<Leader, Rc<RefCell<BasicBlock>>>::new();

        for func in p.functions.iter() {
            let simple_basic_blocks_vec_from_function =
                BasicBlock::simple_basic_blocks_vec_from_function(&func);
            for bb in simple_basic_blocks_vec_from_function {
                let bb_clone = bb.clone();
                hm.insert(bb_clone.borrow().leader.clone(), bb.clone());
            }
        }

        hm
    }

    pub fn print_hm(hm: &HashMap<Leader, Rc<RefCell<BasicBlock>>>) {
        for i in hm.iter() {
            eprintln!("{:?}", i.0);
            eprintln!("{}", i.1.borrow())
        }
    }
}

////#[derive(Debug)]
////struct Instruction {
////    op: String,
////    args: Vec<String>,
////}
//
//type Label = String;
//#[derive(Debug)]
//struct BasicBlock {
//    entry_label: String,
//    instrs: Vec<Instruction>,
//    predecessors: Vec<RefCell<BasicBlock>>, // TODO: needed for cyclic CFGs?
//    successors: Vec<RefCell<BasicBlock>>,
//}
//
//impl BasicBlock {
//    fn new_from_parents(parent_block: RefCell<BasicBlock>) {}
//    fn new_as_entrance() -> Self {
//        BasicBlock {
//            entry_label: "".to_string(),
//            instrs: Vec::new(),
//            predecessors: Vec::new(),
//            successors: Vec::new(),
//        }
//    }
//    //fn vec_from(func: &Function) {
//    //    let first_block = BasicBlock::new(func.name);
//    //}
//    // fn add_predecessor(&mut self, pred: String) {
//    //     self.predecessors.push(pred);
//    // }
//}
//
//#[derive(Debug)]
//struct CFG {
//    function: Function,
//    basic_blocks: Vec<BasicBlock>,
//    entry_block: BasicBlock,
//}
//
//fn generate_set_of_blocks(p: Program) {
//    let basicblock_map = HashMap::<Label, BasicBlock>::default();
//    for func in p.functions {
//        let basic_blocks = BasicBlock::vec_from(&func);
//        basicblock_map.insert
//    }
//}
//impl CFG {
//    fn new(function: String, instructions: Vec<Instruction>) -> Self {
//        let mut basic_blocks = Vec::new();
//
//        let mut future_blocks = Vec::new();
//
//        basic_blocks.push(BasicBlock::new("ENTRY".to_string(), Vec::new()));
//        let entry_block = &mut basic_blocks[0];
//        let mut curr_block = Some(entry_block);
//
//        for inst in instructions {
//            if let Some(op) = inst.op.as_str() {
//                if BRANCH_INSTRUCTIONS.contains(&op) {
//                    curr_block.instrs.push(inst);
//
//                    // control flow aaaa
//                    {
//                        if op == "jmp" {
//                            let succ = BasicBlock::new(inst.args[0].clone(), Vec::new());
//                            curr_block.successors.push(succ);
//                            basic_blocks.push(succ);
//                            curr_block = &mut succ;
//                        } else if op == "br" {
//                            let left = BasicBlock::new(inst.args[1].clone(), Vec::new());
//                            let right = BasicBlock::new(inst.args[2].clone(), Vec::new());
//                            curr_block.successors.push(left);
//                            curr_block.successors.push(right);
//                            basic_blocks.push(left);
//                            basic_blocks.push(right);
//                            future_blocks.push((left, inst.args[0].clone()));
//                            future_blocks.push((right, inst.args[0].clone()));
//                        } else if op == "call" {
//                            basic_blocks
//                                .last_mut()
//                                .unwrap()
//                                .successors
//                                .push(inst.args[0].clone());
//                        } else if op == "ret" {
//                            basic_blocks
//                                .last_mut()
//                                .unwrap()
//                                .successors
//                                .push("EXIT".to_string());
//                        }
//                    }
//                    curr_instrs.clear();
//                }
//            } else if let Some(label) = inst.label.as_ref() {
//                if !curr_instrs.is_empty() {
//                    basic_blocks.push(BasicBlock::new(entry_label.clone(), curr_instrs.clone()));
//                    curr_instrs.clear();
//                }
//                entry_label = label.clone();
//            }
//        }
//
//        // Handle any remaining instructions
//        if !curr_instrs.is_empty() {
//            basic_blocks.push(BasicBlock::new(entry_label, curr_instrs));
//        }
//
//        CFG {
//            function,
//            basic_blocks,
//            entry_block,
//            exit_block,
//        }
//    }
//}
//
//
//impl std::fmt::Display for CFG {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        for bb in &self.basic_blocks {
//            writeln!(f, "{}", bb)?;
//        }
//        Ok(())
//    }
//}
//
//// Example usage
//fn main() {
//    //let function_name = "example_function".to_string();
//    //let instructions = vec![
//    //    Instruction {
//    //        op: "label".to_string(),
//    //        args: vec!["start".to_string()],
//    //    },
//    //    Instruction {
//    //        op: "add".to_string(),
//    //        args: vec!["a".to_string(), "b".to_string()],
//    //    },
//    //    Instruction {
//    //        op: "jmp".to_string(),
//    //        args: vec!["end".to_string()],
//    //    },
//    //    Instruction {
//    //        op: "label".to_string(),
//    //        args: vec!["end".to_string()],
//    //    },
//    //    Instruction {
//    //        op: "ret".to_string(),
//    //        args: vec![],
//    //    },
//    //];
//
//    //let cfg = CFG::new(function_name, instructions);
//    //println!("Control Flow Graph:\n{}", cfg);
//}
