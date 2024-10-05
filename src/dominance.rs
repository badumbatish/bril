use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::cfg::*;
pub struct DominatorTree {}
impl DominatorTree {
    //pub fn new(cfg: cfg::CFG) {}
}
pub struct PessimisticConstProp {}
pub struct DominanceDataFlow {
    pub domset: BTreeMap<usize, BTreeSet<usize>>,
    pub idom: BTreeMap<usize, usize>,
    pub domtree: BTreeMap<usize, usize>,
    pub df: BTreeMap<usize, HashSet<usize>>,
}

impl DominanceDataFlow {
    pub fn new(cfg: &CFG) -> Self {
        // Initially, each node's dominator set is set to the set of all nodes
        let mut result = Self {
            domset: BTreeMap::default(),
            idom: BTreeMap::default(),
            domtree: BTreeMap::default(),
            df: BTreeMap::default(),
        };
        for i in cfg.hm.clone() {
            // INITIALIZE EACH OF THE DOM SET
            let idb = i.1.borrow().id;
            if i.1.borrow().func.is_some() {
                result.domset.entry(idb).or_default().insert(idb);
            } else {
                for ins in 0..cfg.hm.len() {
                    result.domset.entry(idb).or_default().insert(ins);
                }
            }

            // INITIALIZE EACH OF THE DOM TREE
            result.domtree.entry(idb).or_default();
            result.df.entry(idb).or_default();
        }

        result
    }
}

impl DominanceDataFlow {
    pub fn infer(&mut self, cfg: &CFG) -> &mut Self {
        self.infer_idom_set()
            .infer_dom_tree()
            .infer_dominance_frontier(cfg)
    }
    pub fn infer_idom_set(&mut self) -> &mut Self {
        for (block_id, block_dom_set) in self.domset.iter() {
            for potential_candidate in block_dom_set.iter() {
                let mut good_candidate = true;
                if potential_candidate != block_id {
                    for other_candidate in block_dom_set.iter() {
                        if other_candidate == potential_candidate || other_candidate == block_id {
                            continue;
                        }
                        if self
                            .domset
                            .get(other_candidate)
                            .unwrap()
                            .contains(potential_candidate)
                        {
                            //eprintln!(
                            //    "Other: {:?}, potential: {:?}. Disqualified",
                            //    other_candidate, potential_candidate
                            //);
                            good_candidate = false;
                            break;
                        }
                    }
                    if good_candidate {
                        self.idom.insert(*block_id, *potential_candidate);
                        break;
                    } else {
                        continue;
                    }
                }
            }
        }
        // block id 1, block dom set 0 1
        // potential 0
        // if 1's domset contains 0?
        eprintln!("Idom set: {:?}", self.idom);
        self
    }

    /// Always infer the idom set first, then call this method
    // dom_tree[a] = b means b immediately dominates a
    pub fn infer_dom_tree(&mut self) -> &mut Self {
        for (dom, idom) in self.idom.iter() {
            self.domtree.entry(*dom).or_insert(*idom);
        }
        eprintln!("Dom tree: {:?}", self.domtree);
        self
    }

    /// Infer the first two, then call this
    pub fn infer_dominance_frontier(&mut self, cfg: &CFG) -> &mut Self {
        // B in DF[A] if A dominates a predecessor of B, and A doesn't dominate B

        // For all nodes n in the CFG
        //  if n has multiple pred
        //      for each pred of n
        //          runner = p
        //          while runner != idom(n) do
        //          df[runner].insert(n)
        //          runner = idom(runner)
        //
        //

        for (_, node_n) in cfg.hm.iter() {
            if node_n.borrow().predecessors.len() > 1 {
                for pred in node_n.borrow().predecessors.iter() {
                    let mut runner = pred.borrow().id;
                    while *self
                        .domtree
                        .get(&node_n.borrow().id)
                        .unwrap_or(&(runner + 1))
                        != runner
                    {
                        self.df
                            .entry(runner)
                            .or_default()
                            .insert(node_n.borrow().id);

                        runner = self.idom[&runner];
                    }
                }
            }
        }

        eprintln!("Dominance frontier: {:?}", self.df);
        self
    }
}
impl DataFlowAnalysis for DominanceDataFlow {
    fn meet(&mut self, _bb: &mut BasicBlock) {}

    fn transfer(&mut self, bb: &mut BasicBlock) -> TransferResult {
        let initial = self.domset.entry(bb.id).or_default().clone();
        eprintln!("initial of {:?} : {:?}", bb.id, initial);
        let mut pred_id = vec![];

        // Get the ID of predecessors
        for i in bb.predecessors.iter() {
            pred_id.push(i.borrow().id);
        }
        eprintln!("Pred id of {:?} : {:?}", bb.id, pred_id);

        let mut result = BTreeSet::<usize>::new();
        // Take the first set as the initial result
        if pred_id.is_empty() {
            result.insert(bb.id);

            *self.domset.entry(bb.id).or_default() = result.clone();
        } else {
            result = self.domset[pred_id.first().unwrap()].clone();
            for pred in pred_id.iter() {
                result = result
                    .intersection(self.domset.entry(*pred).or_default())
                    .cloned()
                    .collect();
            }
            result.insert(bb.id);
            *self.domset.entry(bb.id).or_default() = result.clone();
        }

        eprintln!("Result {result:?}");
        match initial == result {
            true => TransferResult::NonChanged,
            false => TransferResult::Changed,
        }
    }

    fn transform(&mut self, bb: &mut BasicBlock) {
        eprintln!("Dominator of {:?} : {:?}", bb.id, self.domset.get(&bb.id))
    }

    fn get_dataflow_direction(&self) -> DataFlowDirection {
        DataFlowDirection::Forward
    }

    fn get_dataflow_order(&self) -> DataFlowOrder {
        DataFlowOrder::BFS
    }
}
