use crate::bril_syntax::InstructionOrLabel;
use crate::cfg::CFG;
use crate::{aliases::BbPtr, basic_block::BasicBlock};
use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
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
    Subset(Vec<BbPtr>),
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

impl CFG {
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
        match d.get_dataflow_order() {
            DataFlowOrder::Subset(subset) => self.dataflow_subset(d, &subset),
            _ => self.dataflow_normal(d),
        }
    }
    fn dataflow_subset(&self, d: &mut impl DataFlowAnalysis, subset: &Vec<BbPtr>) {
        let mut q = VecDeque::<BbPtr>::default();
        for bb_ptr in subset {
            q.push_back(bb_ptr.clone());
        }

        let mut changed = true;
        while changed {
            changed = false;
            while !q.is_empty() {
                let visit_bb = q.pop_front().expect("hi").clone();
                // visit_bb.borrow_mut();
                d.meet(&mut visit_bb.borrow_mut());

                if d.transfer(&mut visit_bb.borrow_mut()) == TransferResult::Changed {
                    changed = true;
                }
            }
            if changed {
                for bb_ptr in subset {
                    q.push_back(bb_ptr.clone());
                }
            }
        }
        for i in self.hm.iter() {
            d.transform(&mut i.1.borrow_mut())
        }
    }
    fn dataflow_normal(&self, d: &mut impl DataFlowAnalysis) {
        let mut q = VecDeque::<BbPtr>::default();
        for i in self.hm.clone() {
            if i.1.borrow_mut().func.is_some() {
                match d.get_dataflow_order() {
                    DataFlowOrder::EntryNodesOnly => q.push_back(i.1.clone()),
                    DataFlowOrder::BFS => q.extend(Self::bfs_children(&mut i.1.clone())),
                    DataFlowOrder::PostOrderDFS => todo!(),
                    _ => todo!(),
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
}
