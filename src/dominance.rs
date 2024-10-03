use std::collections::{HashMap, HashSet};

use crate::bril_syntax::{Instruction, InstructionOrLabel, Program};
use crate::cfg::*;
pub struct DominatorTree {}
impl DominatorTree {
    //pub fn new(cfg: cfg::CFG) {}
}
pub struct PessimisticConstProp {}
pub struct DominanceDataFlow {
    pub facts: HashMap<usize, HashSet<usize>>,
}

impl DominanceDataFlow {
    pub fn new(cfg: &CFG) -> Self {
        // Initially, each node's dominator set is set to the set of all nodes
        let mut result = Self {
            facts: HashMap::default(),
        };
        for id in 0..cfg.hm.len() {
            for id2 in 0..cfg.hm.len() {
                result.facts.entry(id).or_default().insert(id2);
            }
        }
        result
    }
}

impl DataFlowAnalysis for DominanceDataFlow {
    fn meet(&mut self, bb: &mut BasicBlock) {}

    fn transfer(&mut self, bb: &mut BasicBlock) -> TransferResult {
        let initial = self.facts.entry(bb.id).or_default().clone();
        let mut pred_id = vec![];

        // Get the ID of predecessors
        for i in bb.predecessors.clone() {
            pred_id.push(i.borrow().id);
        }

        let mut pred_sets = pred_id
            .iter()
            .filter_map(|&id| self.facts.get(&id).cloned());

        // Take the first set as the initial result
        let mut result = pred_sets.next().unwrap_or_default();

        // Intersect with each subsequent set
        for set in pred_sets {
            result = result.intersection(&set).cloned().collect();
        }
        result.insert(bb.id);
        match initial == result {
            true => TransferResult::NonChanged,
            false => TransferResult::Changed,
        }
    }

    fn transform(&mut self, bb: &mut BasicBlock) {}

    fn get_dataflow_direction(&self) -> DataFlowDirection {
        DataFlowDirection::Backward
    }
}
