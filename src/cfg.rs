use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    hash::{Hash, Hasher},
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
pub struct BasicBlock {
    func: Option<Function>,
    pub id: usize,
    pub instrs: Vec<InstructionOrLabel>,
    pub predecessors: Vec<Rc<RefCell<BasicBlock>>>,
    pub successors: Vec<Rc<RefCell<BasicBlock>>>,
}

#[derive(Debug, PartialEq)]
pub enum DataFlowDirection {
    Forward,
    Backward,
}
#[derive(Debug, PartialEq)]
pub enum TransferResult {
    Changed,
    NonChanged,
}
#[derive(Debug, PartialEq)]
pub enum ConditionalTransferResult {
    AllPathTaken,
    FirstPathTaken,
    SecondPathTaken,
    NoPathTaken,
}

pub trait DataFlowAnalysis {
    fn meet(&mut self, bb: &mut BasicBlock);
    fn transfer(&mut self, bb: &mut BasicBlock) -> TransferResult;
    fn transform(&mut self, bb: &mut BasicBlock);
    fn get_dataflow_direction(&self) -> DataFlowDirection;
}

pub trait ConditionalDataFlowAnalysis {
    fn meet(&mut self, bb: &mut BasicBlock);
    fn transfer(&mut self, bb: &mut BasicBlock) -> ConditionalTransferResult;
    fn transform(&mut self, bb: &mut BasicBlock);
}
impl std::fmt::Debug for BasicBlock {
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

impl Hash for BasicBlock {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for BasicBlock {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BasicBlock {}

impl BasicBlock {
    pub fn as_txt_instructions(self) -> String {
        let _result = String::new();
        todo!()
    }
    pub fn default(id: usize) -> BasicBlock {
        Self {
            func: None,
            id,
            instrs: Vec::new(),
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }

    pub fn simple_basic_blocks_vec_from_function(
        f: Function,
        id: &mut usize,
    ) -> Vec<Rc<RefCell<BasicBlock>>> {
        let mut result: Vec<Rc<RefCell<BasicBlock>>> = Vec::new();
        let mut i = 0;
        // let mut last_instruction_before_construction = 0;
        let mut non_linear_before = false;
        while i < f.instrs.len() {
            // this match only happens if instruction is at start of function or after a branch
            // without label
            let b: BasicBlock = BasicBlock::default(*id);
            let bb: Rc<RefCell<BasicBlock>> = Rc::<RefCell<BasicBlock>>::new(b.into());
            *id += 1;
            if result.is_empty() {
                bb.borrow_mut().func = Some(f.clone());
            } else if !non_linear_before {
                bb.borrow_mut()
                    .predecessors
                    .push(result.last().unwrap().clone());
                result
                    .last_mut()
                    .unwrap()
                    .borrow_mut()
                    .successors
                    .push(bb.clone());
                non_linear_before = true;
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
pub struct CFG {
    pub hm: HashMap<InstructionOrLabel, Rc<RefCell<BasicBlock>>>,
}
// main:
// @main
impl CFG {
    pub fn hm_from_function(f: Function) -> HashMap<InstructionOrLabel, Rc<RefCell<BasicBlock>>> {
        // O(n)
        let mut hm = HashMap::<InstructionOrLabel, Rc<RefCell<BasicBlock>>>::new();

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
        let mut hm = HashMap::<InstructionOrLabel, Rc<RefCell<BasicBlock>>>::new();

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
                            //eprintln!("This should not happen in CFG::from_program")
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
            // other_fields: serde_json::Value::default(),
        };

        // The function is here just because we want to maintain the initial order of function in
        // textual IR
        let mut sorted_by_func_id: Vec<Rc<RefCell<BasicBlock>>> =
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

    fn insert_instr_func(&self, bb: Rc<RefCell<BasicBlock>>) -> Function {
        let mut visited = HashSet::<usize>::default();
        let mut q = VecDeque::<Rc<RefCell<BasicBlock>>>::default();
        let mut vec_instr = Vec::<InstructionOrLabel>::new();
        q.push_back(bb.clone());
        visited.insert(bb.borrow().id);
        let mut v = Vec::default();
        v.push(bb.clone());
        while !q.is_empty() {
            let visit_bb = q.pop_front().unwrap();

            for succ in visit_bb.borrow().successors.iter().rev() {
                let a = succ.borrow().id;
                if !visited.contains(&a) {
                    q.push_back(succ.clone());
                    v.push(succ.clone());
                    visited.insert(a);
                }
            }
        }
        v.sort_by_key(|bb| bb.borrow().id);

        for bb in v {
            vec_instr.extend(bb.borrow().instrs.clone());
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
    pub fn dataflow_forward_optimistically(&self, d: &mut impl ConditionalDataFlowAnalysis) {
        // do the dataflow optimistically
        //
        //
        let mut q = VecDeque::<Rc<RefCell<BasicBlock>>>::default();
        for i in self.hm.clone() {
            if i.1.borrow_mut().func.is_some() {
                q.push_back(i.1.clone());
            }
            while !q.is_empty() {
                let visit_bb = q.pop_front().expect("hi").clone();
                // visit_bb.borrow_mut();
                d.meet(&mut visit_bb.borrow_mut());

                let transfer_result = d.transfer(&mut visit_bb.borrow_mut());
                match transfer_result {
                    ConditionalTransferResult::AllPathTaken => {
                        q.extend(visit_bb.borrow_mut().successors.clone())
                    }
                    ConditionalTransferResult::FirstPathTaken => {
                        //eprintln!("First path taken, from {}", s);
                        q.push_back(visit_bb.borrow_mut().successors[0].clone())
                    }
                    ConditionalTransferResult::SecondPathTaken => {
                        q.push_back(visit_bb.borrow_mut().successors[1].clone())
                    }
                    ConditionalTransferResult::NoPathTaken => continue,
                };
            }
        }

        for i in self.hm.iter() {
            d.transform(&mut i.1.borrow_mut())
        }
    }
    pub fn dataflow(&self, d: &mut impl DataFlowAnalysis) {
        // do the dataflow
        //
        //
        let mut q = VecDeque::<Rc<RefCell<BasicBlock>>>::default();
        for i in self.hm.clone() {
            if i.1.borrow_mut().func.is_some() {
                q.extend(Self::bfs_children(&mut i.1.clone()));
            }
            while !q.is_empty() {
                let visit_bb = q.pop_front().expect("hi").clone();
                // visit_bb.borrow_mut();
                d.meet(&mut visit_bb.borrow_mut());

                if d.transfer(&mut visit_bb.borrow_mut()) == TransferResult::Changed {
                    if d.get_dataflow_direction() == DataFlowDirection::Forward {
                        q.extend(visit_bb.borrow_mut().successors.clone());
                    } else if d.get_dataflow_direction() == DataFlowDirection::Backward {
                        q.extend(visit_bb.borrow_mut().predecessors.clone());
                    }
                }
            }
        }

        for i in self.hm.iter() {
            d.transform(&mut i.1.borrow_mut())
        }
    }

    // fn dfs_children(bb: &mut BasicBlock<T>) {}
    fn bfs_children(bb: &mut Rc<RefCell<BasicBlock>>) -> VecDeque<Rc<RefCell<BasicBlock>>> {
        let mut visited = HashSet::<InstructionOrLabel>::default();
        let mut q = VecDeque::<Rc<RefCell<BasicBlock>>>::default();
        let mut result = VecDeque::<Rc<RefCell<BasicBlock>>>::default();

        q.push_back(bb.clone());
        result.push_back(bb.clone());

        visited.insert(bb.borrow().instrs.first().unwrap().clone());
        while !q.is_empty() {
            let visit_bb = q.pop_front().unwrap();
            result.push_back(visit_bb.clone());
            for succ in visit_bb.borrow().successors.iter().rev() {
                let a = succ.borrow().instrs.first().unwrap().clone();
                if !visited.contains(&a) {
                    q.push_back(succ.clone());
                    visited.insert(a);
                }
            }
        }

        result
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
    //
}
