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

            for succ in visit_bb.borrow().successors.iter() {
                let a = succ.borrow().id;
                if !visited.contains(&a) {
                    q.push_back(succ.clone());
                    v.push(succ.clone());
                    visited.insert(a);
                }
            }
        }
        // v.sort_by_key(|bb| bb.borrow().id);

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

    pub fn analyze_loop(&mut self) {
        Loops::new(self);
    }
}

pub struct Loop {
    pub preheader: BbPtr,
    pub header: BbPtr,
    pub latch: BbPtr,
    pub loop_nodes: LinkedList<BbPtr>,
    pub exit: Option<BbPtr>,
}

pub enum PreHeaderCreate {
    Create,
    DontCreate,
}
impl Loop {
    pub fn new_with_header_and_latch(
        cfg: &mut CFG,
        header_id: &BlockID,
        latch_id: &BlockID,
        preheader_create: PreHeaderCreate,
    ) {
        let preheader = match preheader_create {
            PreHeaderCreate::Create => Loop::create_preheader(cfg, header_id),
            PreHeaderCreate::DontCreate => cfg
                .id_to_bb
                .get(header_id)
                .unwrap()
                .borrow()
                .predecessors
                .first()
                .unwrap()
                .clone(),
        };

        // Self{
        //     preheader : preheader,
        //     header : cfg.id_to_bb.get(header_id).unwrap().clone(),
        //     l
        // }
    }

    pub fn create_preheader(cfg: &mut CFG, header_id: &BlockID) -> BbPtr {
        // Create a new BbPtr from cfg's basic block counter
        let label = match cfg
            .id_to_bb
            .get(header_id)
            .unwrap()
            .borrow()
            .instrs
            .front()
            .clone()
            .unwrap()
        {
            InstructionOrLabel::Label(label) => label.label.clone(),
            _ => panic!("Would never happen"),
        };

        let bb = BasicBlock::default_with_label(cfg.basic_block_counter, &(label + "_preheader"));
        eprintln!("{:?}", bb.instrs);
        eprintln!("BB ptr has id {}", bb.id);
        let bb_ptr = BbPtr::new(bb.into());

        // locate the header id

        let header_bb_ptr = cfg.id_to_bb.get(header_id).unwrap();

        let mut bb_ptr_mut = bb_ptr.borrow_mut();
        // bbptr's successor is header id
        bb_ptr_mut.successors.push(header_bb_ptr.clone());
        //

        // All successor of bb_ptr should now point to bb_ptr instead of header_ptr
        //
        for pred in header_bb_ptr.borrow().predecessors.iter() {
            if header_bb_ptr.borrow().id == pred.borrow().id {
                panic!("I detect a self loop here, is this valid for a bril IR ?");
            }
            let pred_id = pred.borrow().id;
            eprintln!("Predecessor {}", pred_id);
            for succ in pred.borrow_mut().successors.iter_mut() {
                if succ.borrow().id != *header_id {
                    continue;
                }

                eprintln!("Predecessor {} has {header_id} before", pred_id);
                *succ = bb_ptr.clone();
            }
            // for succ in pred.borrow().successors.iter() {
            //     if succ.borrow().id != *header_id {
            //         continue;
            //     }
            //
            //     eprintln!("Predecessor {} has {} after", pred_id, succ.borrow().id);
            // }
        }
        // any predecessor of header id is now bbptr's
        bb_ptr_mut
            .predecessors
            .append(&mut header_bb_ptr.borrow_mut().predecessors);
        // header id's only predecessor is bbptr
        header_bb_ptr.borrow_mut().predecessors = Vec::new();
        header_bb_ptr.borrow_mut().predecessors.push(bb_ptr.clone());

        eprintln!("Before, hm has {}", cfg.hm.len());
        // Now, put this to the hashmap before i forgot
        cfg.hm
            .insert(bb_ptr_mut.instrs.front().unwrap().clone(), bb_ptr.clone());
        cfg.id_to_bb.insert(cfg.basic_block_counter, bb_ptr.clone());
        eprintln!("Create a new block with id :  {}", cfg.basic_block_counter);
        cfg.basic_block_counter += 1;

        eprintln!("After, hm has {}", cfg.hm.len());
        bb_ptr.clone()
    }
}
pub struct Loops {
    pub loops: Vec<Loop>,
}

impl Loops {
    fn new(cfg: &mut CFG) {
        let dominance = DominanceDataFlow::new(cfg);
        let mut loop_start_end = BTreeMap::<BlockID, BTreeSet<BlockID>>::new();
        for (dominated, dominator_set) in &dominance.domset {
            match cfg.id_to_bb.get(dominated) {
                Some(bbptr) => {
                    for succ in bbptr.borrow().successors.iter() {
                        let succ_id = &succ.borrow().id;
                        if dominator_set.contains(succ_id) {
                            eprintln!("I see a loop from block {} to block {}", succ_id, dominated);
                            loop_start_end
                                .entry(*succ_id)
                                .or_default()
                                .insert(*dominated);
                        }
                    }
                }
                _ => panic!("This should not happen. All block id should be accounted for"),
            }
        }

        let created_header = BTreeSet::<BlockID>::new();

        let loops = Vec::<Loop>::new();
        for (header_id, latches) in loop_start_end {
            for latch_id in latches {
                let precreate: PreHeaderCreate = match created_header.contains(&header_id) {
                    false => PreHeaderCreate::Create,
                    true => PreHeaderCreate::DontCreate,
                };

                let singly_loop =
                    Loop::new_with_header_and_latch(cfg, &header_id, &latch_id, precreate);
            }
        }
        //for (_, bb_ptr) in cfg.hm.iter() {
        //    let block_id = bb_ptr.borrow().id;
        //    for pred in bb_ptr.borrow().predecessors.iter() {
        //        if dominance.dom(pred.borrow().id, block_id) {
        //            eprintln!(
        //            );
        //        }
        //    }
        //}
    }

    pub fn variable_in_a_loop(&self, _variable_name: String) -> bool {
        todo!()
    }
}
