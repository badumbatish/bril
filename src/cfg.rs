use crate::aliases::{BbPtr, BlockID, IdToBbMap};
use crate::basic_block::BasicBlock;
use crate::bril_syntax::{Function, InstructionOrLabel, Program};
use crate::dominance::DominanceDataFlow;
use std::collections::LinkedList;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    fmt::Debug,
};

#[derive(Debug)]
pub struct CFG {
    pub hm: HashMap<InstructionOrLabel, BbPtr>,
    pub basic_block_counter: BlockID,
    pub id_to_bb: IdToBbMap,
}
// main:
// @main
impl CFG {
    //pub fn components_from_function(
    //    f: Function,
    //) -> (
    //    HashMap<InstructionOrLabel, BbPtr>,
    //    HashMap<BlockID, BbPtr>,
    //) {
    //    // O(n)
    //    let mut hm = HashMap::<InstructionOrLabel, BbPtr>::new();
    //    let mut id_to_bb = HashMap::<BlockID, BbPtr>::new();
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
        let mut hm = HashMap::<InstructionOrLabel, BbPtr>::new();

        let mut id_to_bb = HashMap::<BlockID, BbPtr>::new();

        let mut basic_block_counter: BlockID = 0;
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

        Self {
            hm,
            basic_block_counter,
            id_to_bb,
        }
    }

    pub fn to_program(&self) -> Program {
        let mut p = Program {
            functions: Vec::default(),
            // other_fields: serde_json::Value::default(),
        };

        // The function is here just because we want to maintain the initial order of function in
        // textual IR
        let mut sorted_by_func_id: Vec<BbPtr> = self.hm.values().cloned().collect();

        // Sort the vector by the 'id' field
        sorted_by_func_id.sort_by_key(|e| e.borrow().id);

        for i in sorted_by_func_id {
            if i.borrow().func.is_some() {
                p.functions.push(self.insert_instr_func(i.clone()));
            }
        }
        p
    }

    fn insert_instr_func(&self, bb: BbPtr) -> Function {
        let mut visited = HashSet::<BlockID>::default();
        let mut q = VecDeque::<BbPtr>::default();
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
    //fn make_info_from_block(bb: BbPtr) -> String {
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

pub struct Loop {
    pub preheader: BbPtr,
    pub header: BbPtr,
    pub latch: BbPtr,
    pub loop_nodes: LinkedList<BbPtr>,
    pub exit: BbPtr,
}
pub struct Loops {
    pub loops: Vec<Loop>,
}

impl Loops {
    fn new(cfg: &mut CFG) {
        let dominance = DominanceDataFlow::new(cfg);

        for (_, bb_ptr) in cfg.hm.iter() {
            let block_id = bb_ptr.borrow().id;
            for pred in bb_ptr.borrow().predecessors.iter() {
                if dominance.dom(pred.borrow().id, block_id) {
                    eprintln!(
                        "I see a loop from block {} to block {}",
                        pred.borrow().id,
                        block_id
                    );
                }
            }
        }
    }

    pub fn variable_in_a_loop(&self, variable_name: String) -> bool {
        todo!()
    }
}
