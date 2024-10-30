use crate::aliases::BlockID;
use crate::bril_syntax::InstructionOrLabel;
use crate::cfg::CFG;
use crate::data_flow::DataFlowAnalysis;
use crate::data_flow::DataFlowDirection;
use crate::data_flow::DataFlowOrder;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::LinkedList;

type State = BTreeMap<String, BTreeSet<usize>>;
pub struct AliasAnalysis {
    //
    // A map that stores
    alias_states: BTreeMap<BlockID, State>,

    set_of_all_memory_location: BTreeSet<usize>,
}
impl AliasAnalysis {
    pub fn new(cfg: &CFG) -> AliasAnalysis {
        // TODO: Make a function to query set_of_all_mem from cfg

        let mut alias_states = BTreeMap::<BlockID, State>::default();
        let mut set_of_all_memory_location = BTreeSet::default();
        for (_, bb_ptr) in cfg.hm.iter() {
            alias_states.entry(bb_ptr.borrow().id).or_default();
            for instr in bb_ptr.borrow().instrs.iter() {
                if let crate::bril_syntax::InstructionOrLabel::Instruction(i) = instr {
                    if i.is_alloc() {
                        set_of_all_memory_location.insert(i.instruction_id.unwrap());
                    }
                }
            }
        }
        Self {
            alias_states,
            set_of_all_memory_location,
        }
    }
}
impl DataFlowAnalysis for AliasAnalysis {
    fn meet(&mut self, bb: &mut crate::basic_block::BasicBlock) {
        // if two variables has non-empty intersection, they might alias
        let mut empty_state = State::default();
        //let mut bb_state = self.alias_states.entry(bb.id).or_default();

        for pred in bb.predecessors.iter() {
            let pred_id = pred.borrow().id;
            if pred_id == bb.id {
                continue;
            }

            let pred_state = self.alias_states.get(&pred_id);
            match pred_state {
                Some(pred_state) => {
                    // Union the sets: Extend bb_state with elements from pred_state
                    for (key, values) in pred_state.iter() {
                        empty_state
                            .entry(key.clone())
                            .or_default() // Create a new set if key does not exist
                            .extend(values.iter()); // Add elements from the predecessor's state
                    }
                }
                None => continue,
            }
        }

        let bb_state = self.alias_states.entry(bb.id).or_default();

        for (key, values) in empty_state {
            bb_state.entry(key).or_default().extend(values.iter());
        }
    }

    fn transfer(
        &mut self,
        bb: &mut crate::basic_block::BasicBlock,
    ) -> crate::data_flow::TransferResult {
        let initial_state = self.alias_states.entry(bb.id).or_default().clone();
        let mut changing_state = self.alias_states.entry(bb.id).or_default().clone();
        for instr in bb.instrs.iter() {
            match instr {
                crate::bril_syntax::InstructionOrLabel::Instruction(i) => {
                    if i.is_load() {
                        // INFO: Points x to all mem
                        changing_state
                            .entry(i.dest.clone().unwrap())
                            .or_default()
                            .extend(self.set_of_all_memory_location.clone())
                    } else if i.is_ptradd() | i.is_id() {
                        // INFO: points x to whatever y points to
                        let base_ptr_string = i.args.clone().unwrap().first().unwrap().clone();
                        eprintln!("Base ptr string is {}", base_ptr_string);
                        eprintln!(
                            "State of base ptr is {:?}",
                            self.alias_states
                                .entry(bb.id)
                                .or_default()
                                .entry(base_ptr_string.clone())
                                .or_default()
                                .clone()
                        );
                        *changing_state.entry(i.dest.clone().unwrap()).or_default() =
                            changing_state.entry(base_ptr_string).or_default().clone()
                    } else if i.is_alloc() {
                        // INFO: points x to that location
                        changing_state
                            .entry(i.dest.clone().unwrap())
                            .or_default()
                            .insert(i.instruction_id.unwrap());
                    }
                }
                _ => continue,
            }
        }

        *self.alias_states.entry(bb.id).or_default() = changing_state.clone();
        for (block_id, state) in self.alias_states.iter() {
            eprintln!("Block id: {}", block_id);
            eprintln!("State: {:?}", state)
        }
        match initial_state == changing_state {
            true => crate::data_flow::TransferResult::NonChanged,
            false => crate::data_flow::TransferResult::Changed,
        }
    }

    /// This dead store performs dead store elimination
    fn transform(&mut self, bb: &mut crate::basic_block::BasicBlock) {
        // unused var : map from var(String) to instruction
        // to_be_removed : set of instruction when rescaning again, we don't include this
        //

        // for an instruction in a block
        //  if that instruction uses a variable that points to a memory ->
        //      remove that instruction from unused_var[var]
        //  for every memory location that current thing points to
        //      remove that instruction location from unused_var[var]
        //
        //  set current instruction to unused
        //for (block_id, state) in self.alias_states.iter() {
        //    eprintln!("Block id: {}", block_id);
        //    eprintln!("State: {:?}", state)
        //}
        //eprintln!("Hi!");
        //
        let mut to_be_removed = BTreeMap::<String, usize>::new();
        let mut actually_removed = BTreeSet::<usize>::new();
        let bb_state = self.alias_states.get(&bb.id).unwrap();
        for ilb in bb.instrs.clone() {
            if let InstructionOrLabel::Instruction(i) = ilb {
                if i.is_load() || i.is_ptradd() || i.is_alloc() || i.is_store() || i.is_id() {
                    // for all potential memory location that args points to

                    // If there is anything between two stores. We say that we've used the
                    // store instruction associated to the argument and remove them from the
                    // to be remove
                    if i.is_load() | i.is_ptradd() | i.is_id() {
                        let var = i.args.clone().unwrap().first().unwrap().clone();
                        to_be_removed.remove(&var);
                    }

                    // If it is a store, then we record it,
                    // And if we have seen an unused store instruction associated to the
                    // arugment, we say "ok, we can actually remove it"
                    if i.is_store() {
                        let var = i.args.clone().unwrap().first().unwrap().clone();
                        if to_be_removed.contains_key(&var) {
                            actually_removed.insert(*to_be_removed.get(&var).unwrap());
                        }
                        to_be_removed.insert(var, i.instruction_id.unwrap());
                    }
                    eprintln!("To be removed : {:?}", to_be_removed)
                }
            }
        }
        eprintln!("Actually removed in the end : {:?}", actually_removed);

        let mut kept_instruction = LinkedList::new();
        for ilb in bb.instrs.clone() {
            match ilb {
                InstructionOrLabel::Label(_) => {
                    kept_instruction.push_back(ilb.clone());
                }
                InstructionOrLabel::Instruction(ref instruction) => {
                    if !actually_removed.contains(&instruction.instruction_id.unwrap()) {
                        kept_instruction.push_back(ilb);
                    }
                }
            }
        }

        bb.instrs = kept_instruction;
    }

    fn get_dataflow_direction(&self) -> crate::data_flow::DataFlowDirection {
        DataFlowDirection::Forward
    }

    fn get_dataflow_order(&self) -> crate::data_flow::DataFlowOrder {
        DataFlowOrder::BFS
    }
}
