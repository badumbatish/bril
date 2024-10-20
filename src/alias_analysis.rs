use crate::aliases::BlockID;
use crate::data_flow::DataFlowAnalysis;
use crate::data_flow::DataFlowDirection;
use crate::data_flow::DataFlowOrder;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

pub struct AliasAnalysis {
    //
    // A map that stores
    block_states: BTreeMap<BlockID, BTreeMap<String, BTreeSet<usize>>>,
}

impl DataFlowAnalysis for AliasAnalysis {
    fn meet(&mut self, bb: &mut crate::basic_block::BasicBlock) {
        todo!()
    }

    fn transfer(
        &mut self,
        bb: &mut crate::basic_block::BasicBlock,
    ) -> crate::data_flow::TransferResult {
        todo!()
    }

    fn transform(&mut self, bb: &mut crate::basic_block::BasicBlock) {
        todo!()
    }

    fn get_dataflow_direction(&self) -> crate::data_flow::DataFlowDirection {
        DataFlowDirection::Forward
    }

    fn get_dataflow_order(&self) -> crate::data_flow::DataFlowOrder {
        DataFlowOrder::EntryNodesOnly
    }
}
