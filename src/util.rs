use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    rc::Rc,
};

use crate::bril_syntax::{Function, InstructionOrLabel, Program};

// Maybe this will be useful in the future but for now a leader is the first instruction in the
// block
//#[derive(Hash, Debug, Eq, PartialEq, Clone)]
//pub enum Leader {
//    FunctionName(Function),
//    InstructionOrLabel(InstructionOrLabel),
//}
//
//impl Leader {
//    pub fn from_label_string(label: String) -> Self {
//        Self::InstructionOrLabel(InstructionOrLabel::Label(Label { label }))
//    }
//
//    pub fn from_label(label: Label) -> Self {
//        Self::InstructionOrLabel(InstructionOrLabel::Label(label))
//    }
//
//    pub fn from_instr_or_label(instr: InstructionOrLabel) -> Self {
//        Self::InstructionOrLabel(instr)
//    }
//}

#[derive(Debug)]
pub struct BasicBlock<T> {
    func: Option<Function>,
    id: u32,
    pub instrs: Vec<InstructionOrLabel>,
    pub facts: HashMap<String, T>,
    pub predecessors: Vec<Rc<RefCell<BasicBlock<T>>>>,
    pub successors: Vec<Rc<RefCell<BasicBlock<T>>>>,
}

#[derive(PartialEq)]
pub enum TransferResult {
    CHANGED,
    NonChanged,
}
impl<T: std::fmt::Debug> std::fmt::Display for BasicBlock<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "----Basic Block----
----Instructions: \n",
        )
        .unwrap();
        for instr in self.instrs.iter() {
            writeln!(f, "{:?}", instr).unwrap();
        }
        writeln!(f, "Pred: ").unwrap();
        for instr in self.predecessors.iter() {
            writeln!(f, "{:?}", instr.borrow()).unwrap();
        }
        writeln!(f, "Succ: ").unwrap();
        for instr in self.successors.iter() {
            writeln!(f, "{:?}", instr.borrow()).unwrap();
        }
        writeln!(f, "\n")
    }
}

impl<FactType> BasicBlock<FactType>
where
    FactType: Debug,
{
    pub fn as_txt_instructions(self) -> String {
        let _result = String::new();
        todo!()
    }
    pub fn default(id: u32) -> BasicBlock<FactType> {
        Self {
            func: None,
            id,
            instrs: Vec::new(),
            predecessors: Vec::new(),
            successors: Vec::new(),
            facts: HashMap::new(),
        }
    }

    pub fn simple_basic_blocks_vec_from_function(
        f: Function,
        id: &mut u32,
    ) -> Vec<Rc<RefCell<BasicBlock<FactType>>>> {
        let mut result: Vec<Rc<RefCell<BasicBlock<FactType>>>> = Vec::new();
        let mut i = 0;
        // let mut last_instruction_before_construction = 0;
        let mut non_linear_before = false;
        while i < f.instrs.len() {
            // this match only happens if instruction is at start of function or after a branch
            // without label
            let b: BasicBlock<FactType> = BasicBlock::default(*id);
            let bb: Rc<RefCell<BasicBlock<FactType>>> =
                Rc::<RefCell<BasicBlock<FactType>>>::new(b.into());
            *id += 1;
            if result.is_empty() {
                bb.borrow_mut().func = Some(f.clone());
            } else if !non_linear_before {
                bb.borrow_mut()
                    .successors
                    .push(result.last().unwrap().clone());
                result
                    .last_mut()
                    .unwrap()
                    .borrow_mut()
                    .successors
                    .push(bb.clone());
            }

            let mut bb_mut = bb.borrow_mut();
            bb_mut.instrs.push(f.instrs[i].clone());
            i += 1;
            loop {
                if i >= f.instrs.len() {
                    break;
                }
                match &f.instrs[i] {
                    InstructionOrLabel::Instruction(instr) => {
                        bb_mut
                            .instrs
                            .push(InstructionOrLabel::Instruction(instr.clone()));
                        if instr.is_jmp() || instr.is_br() {
                            non_linear_before = true;
                            break;
                        }
                    }
                    // TODO: Handle doubly label
                    InstructionOrLabel::Label(_) => {
                        i -= 1;
                        non_linear_before = false;
                        break;
                    }
                }
                i += 1;
            }

            result.push(bb.clone());
            i += 1;
        }
        result
    }
}
#[derive(Debug)]
pub struct CFG<T> {
    pub hm: HashMap<InstructionOrLabel, Rc<RefCell<BasicBlock<T>>>>,
}
// main:
// @main
impl<T: std::fmt::Debug> CFG<T> {
    pub fn hm_from_function(
        f: Function,
    ) -> HashMap<InstructionOrLabel, Rc<RefCell<BasicBlock<T>>>> {
        // O(n)
        let mut hm = HashMap::<InstructionOrLabel, Rc<RefCell<BasicBlock<T>>>>::new();

        let mut basic_block_counter = 0;
        let simple_basic_blocks_vec_from_function =
            BasicBlock::simple_basic_blocks_vec_from_function(f, &mut basic_block_counter);
        for bb in simple_basic_blocks_vec_from_function {
            let bb_clone = bb.clone();
            hm.insert(
                bb_clone.borrow().instrs.first().unwrap().clone(),
                bb.clone(),
            );
        }

        hm
        // O(n)
    }
    pub fn from_program(p: Program) -> Self {
        let mut hm = HashMap::<InstructionOrLabel, Rc<RefCell<BasicBlock<T>>>>::new();

        let mut basic_block_counter = 0;
        // Iterate to put basic blocks into the graph
        for func in p.functions {
            let simple_basic_blocks_vec_from_function =
                BasicBlock::simple_basic_blocks_vec_from_function(func, &mut basic_block_counter);
            for bb in simple_basic_blocks_vec_from_function {
                hm.insert(bb.borrow().instrs.first().unwrap().clone(), bb.clone());
            }
        }
        // Iterate to connect them
        for i in hm.values() {
            let mut bi = i.borrow_mut();
            if !bi.instrs.is_empty() {
                match bi.instrs.clone().last() {
                    Some(instr) => match instr {
                        InstructionOrLabel::Label(_) => {
                            eprintln!("This should not happen in CFG::from_program")
                        }
                        InstructionOrLabel::Instruction(ins) => {
                            if ins.is_br() {
                                bi.successors.push(
                                    hm[&InstructionOrLabel::from(
                                        ins.labels.clone().unwrap()[1].clone(),
                                    )]
                                        .clone(),
                                );
                                bi.successors.push(
                                    hm[&InstructionOrLabel::from(
                                        ins.labels.clone().unwrap()[0].clone(),
                                    )]
                                        .clone(),
                                );

                                hm[&InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[1].clone(),
                                )]
                                    .borrow_mut()
                                    .predecessors
                                    .push(i.clone());
                                hm[&InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[0].clone(),
                                )]
                                    .borrow_mut()
                                    .predecessors
                                    .push(i.clone());
                            } else if ins.is_jmp() {
                                bi.successors.push(
                                    hm[&InstructionOrLabel::from(
                                        ins.labels.clone().unwrap()[0].clone(),
                                    )]
                                        .clone(),
                                );
                                hm[&InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[0].clone(),
                                )]
                                    .borrow_mut()
                                    .predecessors
                                    .push(i.clone());
                            }
                        }
                    },
                    None => continue,
                }
            }
        }

        Self { hm }
    }

    pub fn to_program(&self) -> Program {
        let mut p = Program {
            functions: Vec::default(),
            other_fields: serde_json::Value::default(),
        };

        // The function is here just because we want to maintain the initial order of function in
        // textual IR
        let mut sorted_by_func_id: Vec<Rc<RefCell<BasicBlock<T>>>> =
            self.hm.values().cloned().collect();

        // Sort the vector by the 'id' field
        sorted_by_func_id.sort_by_key(|e| e.borrow().id);

        for i in sorted_by_func_id {
            if i.borrow().func.is_some() {
                p.functions.push(self.insert_instr_func(i.clone()));
            }
        }
        p
    }

    fn insert_instr_func(&self, bb: Rc<RefCell<BasicBlock<T>>>) -> Function {
        let mut visited = HashSet::<InstructionOrLabel>::default();

        let mut q = VecDeque::<Rc<RefCell<BasicBlock<T>>>>::default();

        let mut vec_instr = Vec::<InstructionOrLabel>::new();
        q.push_back(bb.clone());

        while !q.is_empty() {
            let visit_bb = q.pop_front().unwrap();

            vec_instr.extend(visit_bb.borrow().instrs.clone());
            visited.insert(visit_bb.borrow().instrs.first().unwrap().clone());

            for succ in visit_bb.borrow().successors.iter() {
                let a = succ.borrow().instrs.first().unwrap().clone();
                if !visited.contains(&a) {
                    q.push_back(succ.clone());
                }
            }
        }

        let mut func = bb.borrow().func.clone().unwrap().clone();
        func.instrs = vec_instr;

        func
    }

    pub fn print_hm(self) {
        for i in self.hm.iter() {
            eprintln!("{:?}", i.0);
            eprintln!("{:?}", i.1.borrow())
        }
    }

    pub fn dataflow_forward<F1, F2, F3>(
        &self,
        mut meet_func: F1,
        mut transfer_func: F2,
        mut optimize_func: F3,
    ) where
        F1: FnMut(&mut BasicBlock<T>),
        F2: FnMut(&mut BasicBlock<T>) -> TransferResult,
        F3: FnMut(&mut BasicBlock<T>),
    {
        // do the dataflow
        //
        //
        let mut q = VecDeque::<Rc<RefCell<BasicBlock<T>>>>::default();
        for i in self.hm.clone() {
            if i.1.borrow_mut().func.is_some() {
                q.push_back(i.1.clone());
            }
            while !q.is_empty() {
                let visit_bb = q.pop_front().expect("hi").clone();
                // visit_bb.borrow_mut();
                meet_func(&mut visit_bb.borrow_mut());

                if transfer_func(&mut visit_bb.borrow_mut()) == TransferResult::CHANGED {
                    q.extend(visit_bb.borrow_mut().successors.clone());
                }
            }
        }

        for i in self.hm.iter() {
            optimize_func(&mut i.1.borrow_mut())
        }
    }

    //pub fn to_dot_string(&self) -> String {
    //    let mut graph_as_string = String::from("digraph {\n");
    //
    //    for i in self.hm.iter() {
    //        let node = format!(
    //            "{} [label=\"{} \"] \n",
    //            Self::make_node_name(*i.0),
    //            Self::make_info_from_block(i.1.clone())
    //        );
    //        graph_as_string.push_str(&node);
    //    }
    //
    //    for i in self.hm.iter() {
    //        graph_as_string += &Self::make_rel_from_block(i.1.clone())
    //    }
    //    // Setting up edge for the nodes
    //
    //    graph_as_string.push_str("}");
    //
    //    graph_as_string
    //}
    //fn make_node_name(id: u32) -> String {
    //    "\tnode".to_owned() + &id.to_string()
    //}
    //fn make_info_from_block(bb: Rc<RefCell<BasicBlock>>) -> String {
    //    let mut result = String::new();
    //
    //    match &bb.borrow().leader {
    //        Leader::FunctionName(fn_name) => {
    //            result.push_str(&("@".to_owned() + &fn_name.clone() + "(...) \\n"));
    //            for i in bb.borrow().instrs.iter().skip(1) {
    //                result += &(i.to_string() + "\\n");
    //            }
    //        }
    //        Leader::InstructionOrLabel(_) => {
    //            for i in bb.borrow().instrs.iter() {
    //                result += &(i.to_string() + "\\n");
    //            }
    //        }
    //    }
    //
    //    result
    //}
    //
    //fn make_rel_from_block(bb: Rc<RefCell<BasicBlock>>) -> String {
    //    let mut result = String::from(Self::make_node_name(bb.borrow().id) + "-> { ");
    //
    //    for pred in &bb.borrow().predecessors {
    //        result += &(Self::make_node_name(pred.borrow().id) + &" ".to_string());
    //    }
    //
    //    for succ in &bb.borrow().successors {
    //        result += &(Self::make_node_name(succ.borrow().id) + &" ".to_string());
    //    }
    //    result += " }\n";
    //    result
    //}
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
