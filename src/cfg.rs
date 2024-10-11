use crate::aliases::{BlockID, IdToBbMap};
use crate::basic_block::BasicBlock;
use crate::bril_syntax::{Function, InstructionOrLabel, Program};
use crate::dominance::DominanceDataFlow;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    fmt::Debug,
    rc::Rc,
};

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
#[derive(Debug, PartialEq)]
pub enum DataFlowOrder {
    EntryNodesOnly,
    PostOrderDFS,
    BFS,
}
pub trait DataFlowAnalysis {
    fn meet(&mut self, bb: &mut BasicBlock);
    fn transfer(&mut self, bb: &mut BasicBlock) -> TransferResult;
    fn transform(&mut self, bb: &mut BasicBlock);
    fn get_dataflow_direction(&self) -> DataFlowDirection;
    fn get_dataflow_order(&self) -> DataFlowOrder;
}

pub trait ConditionalDataFlowAnalysis {
    fn meet(&mut self, bb: &mut BasicBlock);
    fn transfer(&mut self, bb: &mut BasicBlock) -> ConditionalTransferResult;
    fn transform(&mut self, bb: &mut BasicBlock);
}

#[derive(Debug)]
pub struct CFG {
    pub hm: HashMap<InstructionOrLabel, Rc<RefCell<BasicBlock>>>,
    pub id_to_bb: IdToBbMap,
}
// main:
// @main
impl CFG {
    //pub fn components_from_function(
    //    f: Function,
    //) -> (
    //    HashMap<InstructionOrLabel, Rc<RefCell<BasicBlock>>>,
    //    HashMap<BlockID, Rc<RefCell<BasicBlock>>>,
    //) {
    //    // O(n)
    //    let mut hm = HashMap::<InstructionOrLabel, Rc<RefCell<BasicBlock>>>::new();
    //    let mut id_to_bb = HashMap::<BlockID, Rc<RefCell<BasicBlock>>>::new();
    //    let mut basic_block_counter = 0;
    //    let simple_basic_blocks_vec_from_function =
    //        BasicBlock::simple_basic_blocks_vec_from_function(f, &mut basic_block_counter);
    //
    //    for bb in simple_basic_blocks_vec_from_function {
    //        let bb_clone = bb.clone();
    //        hm.insert(
    //            bb_clone.borrow().instrs.front().unwrap().clone(),
    //            bb.clone(),
    //        );
    //        id_to_bb.insert(bb.borrow().id, bb.clone());
    //    }
    //
    //    (hm, id_to_bb)
    //    // O(n)
    //}
    pub fn from_program(p: Program) -> Self {
        let mut hm = HashMap::<InstructionOrLabel, Rc<RefCell<BasicBlock>>>::new();

        let mut id_to_bb = HashMap::<BlockID, Rc<RefCell<BasicBlock>>>::new();
        let mut basic_block_counter = 0;
        // Iterate to put basic blocks into the graph
        for func in p.functions {
            let simple_basic_blocks_vec_from_function =
                BasicBlock::simple_basic_blocks_vec_from_function(func, &mut basic_block_counter);
            for bb in simple_basic_blocks_vec_from_function {
                hm.insert(bb.borrow().instrs.front().unwrap().clone(), bb.clone());
                id_to_bb.insert(bb.borrow().id, bb.clone());
            }
        }
        // Iterate to connect them
        for i in hm.clone().into_values() {
            let mut bi = i.borrow_mut();
            if !bi.instrs.is_empty() {
                match bi.instrs.clone().into_iter().last() {
                    Some(instr) => match instr {
                        InstructionOrLabel::Label(_) => {
                            //eprintln!("This should not happen in CFG::from_program")
                        }
                        InstructionOrLabel::Instruction(ins) => {
                            if ins.is_br() {
                                let first_branch_label = &InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[1].clone(),
                                );
                                let second_branch_label = &InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[0].clone(),
                                );
                                bi.successors.push(hm[first_branch_label].clone());
                                hm[first_branch_label]
                                    .borrow_mut()
                                    .predecessors
                                    .push(i.clone());

                                bi.successors.push(hm[second_branch_label].clone());
                                hm[second_branch_label]
                                    .borrow_mut()
                                    .predecessors
                                    .push(i.clone());
                                //bi.successors.push(
                                //    hm[&InstructionOrLabel::from(
                                //        ins.labels.clone().unwrap()[1].clone(),
                                //    )]
                                //        .clone(),
                                //);
                                //bi.successors.push(
                                //    hm[&InstructionOrLabel::from(
                                //        ins.labels.clone().unwrap()[0].clone(),
                                //    )]
                                //        .clone(),
                                //);
                                //
                                //hm[&InstructionOrLabel::from(
                                //    ins.labels.clone().unwrap()[1].clone(),
                                //)]
                                //    .borrow_mut()
                                //    .predecessors
                                //    .push(i.clone());
                                //hm[&InstructionOrLabel::from(
                                //    ins.labels.clone().unwrap()[0].clone(),
                                //)]
                                //    .borrow_mut()
                                //    .predecessors
                                //    .push(i.clone());
                            } else if ins.is_jmp() {
                                let first_branch_label = &InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[0].clone(),
                                );
                                bi.successors.push(hm[first_branch_label].clone());
                                hm[first_branch_label]
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

        Self { hm, id_to_bb }
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
        let mut visited = HashSet::<BlockID>::default();
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

    pub fn print_hm(&self) {
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
                match d.get_dataflow_order() {
                    DataFlowOrder::EntryNodesOnly => q.push_back(i.1.clone()),
                    DataFlowOrder::BFS => q.extend(Self::bfs_children(&mut i.1.clone())),
                    DataFlowOrder::PostOrderDFS => todo!(),
                }
                while !q.is_empty() {
                    let visit_bb = q.pop_front().expect("hi").clone();
                    // visit_bb.borrow_mut();
                    d.meet(&mut visit_bb.borrow_mut());

                    if d.transfer(&mut visit_bb.borrow_mut()) == TransferResult::Changed {
                        match d.get_dataflow_direction() {
                            DataFlowDirection::Forward => {
                                q.extend(visit_bb.borrow_mut().successors.clone())
                            }
                            DataFlowDirection::Backward => {
                                q.extend(visit_bb.borrow_mut().predecessors.clone())
                            }
                        }
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

        visited.insert(bb.borrow().instrs.front().unwrap().clone());
        while !q.is_empty() {
            let visit_bb = q.pop_front().unwrap();
            result.push_back(visit_bb.clone());
            for succ in visit_bb.borrow().successors.iter().rev() {
                let a = succ.borrow().instrs.front().unwrap().clone();
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

/// INFO: This impl block is denoted to be about SSA
impl CFG {
    /// A map that maps a variable to all the block that defines it
    pub fn def_variables(&mut self) -> BTreeMap<String, BTreeSet<BlockID>> {
        // For each blocks,
        //   For each def in each block
        //     Let a particular def be about v
        //     Add def[v].insert(block)
        let mut result = BTreeMap::<String, BTreeSet<BlockID>>::new();
        for (_, bbrc) in self.hm.iter() {
            for d in bbrc.borrow().get_definitions() {
                if let InstructionOrLabel::Instruction(i) = d {
                    result
                        .entry(i.dest.clone().unwrap())
                        .or_default()
                        .insert(bbrc.borrow().id);
                } else {
                    continue;
                };
            }
        }
        result
    }
    pub fn place_phi_functions_and_generate_ssa(&mut self) {
        let defs = self.def_variables();

        let mut dff = DominanceDataFlow::new(self);
        self.dataflow(&mut dff);
        let df = dff.infer(self).df.clone();

        // INFO: A function to place phi functions down
        let place_phi_functions = || {
            for (var, defs_of_var) in defs.iter() {
                for defining_block in defs_of_var.iter() {
                    for block in df[defining_block].iter() {
                        let label = self.id_to_bb[defining_block]
                            .borrow()
                            .instrs
                            .front()
                            .unwrap()
                            .clone();
                        if self.id_to_bb[block]
                            .borrow()
                            .contains_phi_def(var, label.clone())
                        {
                            continue;
                        } else {
                            let mut block_mut_b = self.id_to_bb[block].borrow_mut();

                            block_mut_b.insert_phi_def(var, label);
                        }
                    }
                }
            }
        };
        place_phi_functions();

        let mut stack_of = BTreeMap::<String, Vec<String>>::new();
        for (def, _) in defs.iter() {
            stack_of.entry(def.clone()).or_insert(vec![def.clone()]);
        }
        let mut name_counter = BTreeMap::<String, usize>::new();

        // INFO: A function to rename operands in phi functions
        let mut rename_phi_defs = |stack_of: BTreeMap<String, Vec<String>>| {
            for (_, bb) in self.hm.iter() {
                if bb.borrow().func.is_none() {
                    continue;
                }
                bb.borrow_mut().rename_phi_def(
                    stack_of.clone(),
                    &dff.domtree,
                    &mut name_counter,
                    &self.id_to_bb,
                )
            }
        };
        rename_phi_defs(stack_of);
    }
}
