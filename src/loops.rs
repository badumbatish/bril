use std::collections::{BTreeMap, BTreeSet, LinkedList, VecDeque};

use crate::{
    aliases::{BbPtr, BlockID},
    basic_block::BasicBlock,
    bril_syntax::InstructionOrLabel,
    cfg::CFG,
    data_flow::{DataFlowAnalysis, TransferResult},
    dominance::DominanceDataFlow,
};

pub struct Loop {
    pub preheader: BbPtr,
    pub header: BbPtr,
    pub latch: BbPtr,
    pub loop_nodes: VecDeque<BbPtr>,
    pub exit: Option<BbPtr>,
    pub defined_variables: BTreeSet<String>,

    // INFO: Dataflow information
    pub invariant_variable_maps: BTreeMap<usize, BTreeSet<String>>,
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
    ) -> Loop {
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
        let loop_nodes = Self::bfs_from_latches_to_head(cfg, header_id, latch_id);

        let mut result = Self {
            preheader,
            header: cfg.id_to_bb.get(header_id).unwrap().clone(),
            latch: cfg.id_to_bb.get(latch_id).unwrap().clone(),
            loop_nodes,
            exit: None,
            defined_variables: BTreeSet::default(),
            invariant_variable_maps: BTreeMap::default(),
        };

        for node in result.loop_nodes.iter() {
            result
                .invariant_variable_maps
                .entry(node.borrow().id)
                .or_default();
        }
        result.defined_variables = result.get_defined_variables();

        result
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
            .unwrap()
        {
            InstructionOrLabel::Label(label) => label.label.clone(),
            _ => panic!("Would never happen"),
        };

        let internal_counter = cfg.basic_block_counter;
        let bb =
            BasicBlock::default_with_label(&mut cfg.basic_block_counter, &(label + "_preheader"));

        eprintln!("{:?}", bb.instrs);
        eprintln!("BB ptr has id {}", bb.id);
        let bb_ptr = BbPtr::new(bb.into());

        // locate the header id

        let header_bb_ptr = cfg.id_to_bb.get(header_id).unwrap();
        eprintln!(
            "Current header is {:?}",
            header_bb_ptr.borrow().instrs.front()
        );
        // bbptr's successor is header id
        bb_ptr.borrow_mut().successors.push(header_bb_ptr.clone());
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
        bb_ptr
            .borrow_mut()
            .predecessors
            .append(&mut header_bb_ptr.borrow_mut().predecessors);
        // header id's only predecessor is bbptr
        header_bb_ptr.borrow_mut().predecessors = Vec::new();
        header_bb_ptr.borrow_mut().predecessors.push(bb_ptr.clone());

        eprintln!("Before, hm has {}", cfg.hm.len());
        // Now, put this to the hashmap before i forgot
        cfg.hm.insert(
            bb_ptr.borrow_mut().instrs.front().unwrap().clone(),
            bb_ptr.clone(),
        );
        cfg.id_to_bb.insert(internal_counter, bb_ptr.clone());

        //let mut tail = self.instrs.split_off(position);
        //
        //self.instrs.push_back(ilb.clone()); // Manually walk the iterator to the desired position
        //self.instrs.append(&mut tail);
        let mut i: usize = 0;
        for bb_ptr in cfg.bb_ptr_vec.iter() {
            if bb_ptr.borrow().id == *header_id {
                break;
            }
            i += 1;
        }
        let mut tail = cfg.bb_ptr_vec.split_off(i);
        cfg.bb_ptr_vec.push_back(bb_ptr.clone());

        cfg.bb_ptr_vec.append(&mut tail);
        eprintln!("Create a new block with id :  {}", cfg.basic_block_counter);

        eprintln!("After, hm has {}", cfg.hm.len());
        bb_ptr.clone()
    }

    pub fn get_defined_variables(&self) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        for node in self.loop_nodes.iter() {
            for instr in node.borrow().instrs.clone() {
                match instr {
                    InstructionOrLabel::Instruction(i) => {
                        if i.dest.is_some() {
                            result.insert(i.dest.unwrap());
                        }
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        result
    }
    ///  We basically bfs from the latches up to the header
    fn bfs_from_latches_to_head(
        cfg: &mut CFG,
        header_id: &BlockID,
        latch_id: &BlockID,
    ) -> VecDeque<BbPtr> {
        let header_bb = cfg.id_to_bb.get(header_id).unwrap();
        let latch_bb = cfg.id_to_bb.get(latch_id).unwrap();

        let mut q = VecDeque::new();
        q.push_back(latch_bb.clone());

        let mut loop_nodes = VecDeque::new();
        loop_nodes.push_back(latch_bb.clone());

        let mut visited = BTreeSet::<BlockID>::new();
        visited.insert(latch_bb.borrow().id);

        eprintln!("Putting in the back the latch : {}", latch_id);
        while !q.is_empty() {
            let a = q.pop_front().unwrap();

            if a.borrow().id == *header_id {
                continue;
            }

            for preq in a.borrow().predecessors.clone() {
                if !visited.contains(&preq.borrow().id) && preq.borrow().id != *header_id {
                    q.push_front(preq.clone());
                    loop_nodes.push_front(preq.clone());
                }
            }
        }
        loop_nodes.push_front(header_bb.clone());
        eprintln!("From start to finish");
        for node in loop_nodes.iter() {
            eprintln!("Node {}", node.borrow().id);
        }

        loop_nodes
    }
}
impl Loop {
    fn register_variable(&mut self, bb_id: BlockID, var: &String) -> bool {
        if !self.defined_variables.contains(var) {
            true
        } else {
            self.invariant_variable_maps
                .entry(bb_id)
                .or_default()
                .contains(var)
        }
    }
}
impl DataFlowAnalysis for Loop {
    fn meet(&mut self, bb: &mut BasicBlock) {
        // TODO: For all predecessor, union them with the current basic block's facts

        let mut keys = BTreeSet::<usize>::new();
        keys.insert(bb.id);
        for pred in bb.predecessors.iter() {
            keys.insert(pred.borrow().id);
        }

        *self.invariant_variable_maps.entry(bb.id).or_default() =
            keys.iter().fold(BTreeSet::new(), |acc, &key| {
                if let Some(set) = self.invariant_variable_maps.get(&key) {
                    acc.union(set).cloned().collect()
                } else {
                    acc
                }
            });
    }

    fn transfer(&mut self, bb: &mut BasicBlock) -> crate::data_flow::TransferResult {
        //  A value (we are in SSA!) is loop invariant if either:
        let clone_state = self
            .invariant_variable_maps
            .entry(bb.id)
            .or_default()
            .clone();
        // It is defined outside the loop
        // INFO: For this, check against self.defined_variable
        for ilb in bb.instrs.iter() {
            match ilb {
                InstructionOrLabel::Instruction(i) => {
                    let mut all_arg_invariant = true;
                    if let Some(args) = &i.args {
                        for arg in args.iter() {
                            if self.register_variable(bb.id, arg) {
                                self.invariant_variable_maps
                                    .entry(bb.id)
                                    .or_default()
                                    .insert(arg.clone());
                            } else {
                                all_arg_invariant = false;
                            }
                        }
                    }

                    if all_arg_invariant || i.is_const() {
                        if let Some(dest) = &i.dest {
                            self.invariant_variable_maps
                                .entry(bb.id)
                                .or_default()
                                .insert(dest.clone());
                        }
                    }
                }

                InstructionOrLabel::Label(_) => {}
            }
        }
        // It is defined inside the loop, and:
        // All arguments to the instruction are loop invariant
        // The instruction is deterministic
        // INFO: For this, if it is LICM, put it in invariant_variable_maps

        match clone_state
            == self
                .invariant_variable_maps
                .entry(bb.id)
                .or_default()
                .clone()
        {
            true => TransferResult::NonChanged,
            false => TransferResult::Changed,
        }
    }

    fn transform(&mut self, bb: &mut BasicBlock) {
        // TODO: All the loop invariant code (in terms of definition), we move to preheader
        //
        let mut kept_instruction = LinkedList::<InstructionOrLabel>::new();
        let var_map = self
            .invariant_variable_maps
            .entry(bb.id)
            .or_default()
            .clone();
        for ilb in bb.instrs.iter() {
            match ilb {
                InstructionOrLabel::Instruction(i) => {
                    if let Some(dest) = &i.dest {
                        // if it is defined in the loop and it is loop invariant
                        if self.defined_variables.contains(dest) && var_map.contains(dest) {
                            self.preheader.borrow_mut().instrs.push_back(ilb.clone())
                        } else {
                            kept_instruction.push_back(ilb.clone())
                        }
                    } else {
                        kept_instruction.push_back(ilb.clone())
                    }
                }

                InstructionOrLabel::Label(_) => kept_instruction.push_back(ilb.clone()),
            }
        }

        bb.instrs = kept_instruction;
    }

    fn get_dataflow_direction(&self) -> crate::data_flow::DataFlowDirection {
        panic!()
    }

    fn get_dataflow_order(&self) -> crate::data_flow::DataFlowOrder {
        crate::data_flow::DataFlowOrder::Subset(self.loop_nodes.clone())
    }
}
pub struct Loops {
    pub loops: Vec<Loop>,
}

impl Loops {
    pub fn new(cfg: &mut CFG) -> Loops {
        let dominance = DominanceDataFlow::new(cfg);
        let mut loop_start_end = BTreeMap::<BlockID, BTreeSet<BlockID>>::new();
        for (dominated, dominator_set) in &dominance.domset {
            match cfg.id_to_bb.get(dominated) {
                Some(bbptr) => {
                    for succ in bbptr.borrow().successors.iter() {
                        let succ_id = &succ.borrow().id;
                        if dominator_set.contains(succ_id)
                            && (cfg.id_to_bb[succ_id].borrow().ends_with_br()
                                || cfg.id_to_bb[succ_id].borrow().ends_with_br())
                        {
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

        let mut created_header = BTreeSet::<BlockID>::new();

        let mut loops = Vec::<Loop>::new();

        // Now, given our header and our latch, we can construct a loop by first finding all the
        // loop nodes.
        //
        // then add them together
        for (header_id, latches) in loop_start_end {
            for latch_id in latches {
                let precreate: PreHeaderCreate = match created_header.contains(&header_id) {
                    false => PreHeaderCreate::Create,
                    true => PreHeaderCreate::DontCreate,
                };
                created_header.insert(header_id);

                loops.push(Loop::new_with_header_and_latch(
                    cfg, &header_id, &latch_id, precreate,
                ));
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
        //
        Self { loops }
    }
}
