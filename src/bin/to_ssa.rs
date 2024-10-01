use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use bril::bril_syntax::{BrilType, Instruction, Label, InstructionOrLabel, Program};
use bril::cfg::{BasicBlock, ConditionalTransferResult, CFG};
use serde_json::Value;

#[derive(Debug)]

pub struct VNTable<T> {
    // Since Rust is strict about matching types and doesn't allow a custom Phi object to inherit 
    // from Instruction, use numbers to represent instructions until substitution time
    // current_def and num_to_val chained together roughly approximates current_def in the paper

    var_to_num: HashMap<String, HashMap<usize, usize>>, // Variable to block_id, to value number in that block
    num_to_val: Vec<Instruction>,   // nonlocal value number to instruction. Note that `dest` is unused at this point
    sealed_blocks: HashSet<usize>,  // map of block_ids where block is sealed
    phi_to_block: HashMap<Instruction, usize>, // map of phi instructions to id of containing block
    incomplete_phis: HashMap<usize, HashMap<String, Instruction>>, // block_id to 
}

impl<T> VNTable<T> {
    // Local Value Numbering
    pub fn write_variable(&mut self, var: String, block: BasicBlock<T>, val: Instruction) {
        // TODO: slight inefficiency as clobbered variables aren't removed from the table 
        // all in all, it might be safe to assume num clobbered variables < overhead from hashmap
        // we could run experiments to verify
        // Also, we don't want to reuse the value number because doing so would violate 
        // the single assignment property
        let var_blocks = self.current_def.entry(var.clone()).or_insert_with(HashMap::new);
        var_blocks.insert(block, self.num_to_val.len());
        self.num_to_val.push(val);
    }

    // Rather than return the instruction, return the value number. 
    // Required for making phis work in Bril's syntax. 
    pub fn read_variable(&self, var: String, block: BasicBlock<T>) -> usize {
        if self.current_def.contains_key(&var) {
            // Defined within block; LVN scenario
            return Some(Some(self.current_def.get(&var)).expect("Impossible case reached!").get(&block)).expect("Impossible case reached!");
        }
        // Look for definition in predecessors
        return self.read_variable_recursive(&var, &block);
    }

    // Global Value Numbering
    pub fn read_variable_recursive(&mut self, var: String, block: BasicBlock<T>) -> usize {
        let mut val; 
        if !self.sealed_blocks.contains(&block) {
            // Incomplete CFG 
            val = self.new_phi(&block);
            self.incomplete_phis.get(&block).insert(var.clone(), &val);
        } else if block.predecessors.len() == 1 {
            // Trivial phi; can optimize
            let val_num = self.read_variable(var, block.predecessors.first());
        } else {
            val = self.new_phi(&block);
            self.write_variable(var, block, val);
            self.add_phi_operands(var, val);
        }
        self.write_variable(var, block, val);
        return val;
    }

    pub fn new_phi(&mut self, block: &BasicBlock<T>) -> Instruction {
        let phi = Instruction {
            op: "phi".to_string(),
            dest: None,
            args: Some(Vec::new()),
            labels: Some(Vec::new()),
            bril_type: None,
            value: None,
            funcs: None,
        };
        self.phi_to_block.insert(&phi, &block);
        return phi;
    }

    pub fn append_phi_operand(self, phi: Instruction, label: String, val: String) {
        assert!(&phi.is_phi());
        phi.args.unwrap_or_else(|| Vec::new()).push(val);
        phi.labels.unwrap_or_else(|| Vec::new()).push(label);
        // let mut args = phi.args.unwrap_or_else(|| Vec::new());
        // args.push(val);
        // let mut labels = phi.labels.unwrap_or_else(|| Vec::new());
        // labels.push(label);
    }

    //TODO: 
    pub fn add_phi_operands(&mut self, var: String, phi: Instruction) -> Instruction {
        for pred in self.phi_to_block.get(&phi) {
            let binding = self.read_variable_recursive(var, pred);
            self.append_phi_operand(phi, pred, binding);
        }
        return self.try_trivial_remove_phi(phi);
    }

    pub fn try_trivial_remove_phi(self, phi: Instruction) -> Instruction {
        // for now, try to get everything else to work
        return phi;
    }

    pub fn seal_block(&mut self, block: BasicBlock<T>) {
        for var in self.incomplete_phis.get(&block) {
            self.add_phi_operands(var, self.incomplete_phis.get(&block).get(&var));
        }
        self.sealed_blocks.insert(block);
    }

    // TODO: remove redundant phis using SCCs
    

}




fn main() {
    let prog = Program::stdin();

    let cfg = CFG::from_program(prog);

    // cfg.dataflow_forward_optimistically(forward_meet, forward_transfer, transform);
    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
