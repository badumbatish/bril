use std::collections::{HashMap, LinkedList};

use bril::basic_block::BasicBlock;
use bril::bril_syntax::{Instruction, InstructionOrLabel, Program};
use bril::cfg::CFG;
use bril::data_flow::{DataFlowAnalysis, DataFlowDirection, DataFlowOrder, TransferResult};
#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
pub enum LatticeValue {
    Alisa,
    Dead,
    StrongAlisa,
}

struct LivenessAnalysis {
    pub facts: HashMap<usize, HashMap<String, LatticeValue>>,
}
impl LivenessAnalysis {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
        }
    }
    // Combine lattice value based on the lattice value type
    /// This is called in a meet function on each instruction
    pub fn lattice_value_meet(
        &self,
        q: Option<&LatticeValue>,
        p: Option<&LatticeValue>,
    ) -> LatticeValue {
        // We don't dare delete a value if it is not dead yet, let's be conservative
        // eprintln!("Meet of {:?} in {:?}", l.0, bb.instrs.first());
        match (q, p) {
            (Some(a), Some(b)) => match (a, b) {
                (LatticeValue::Alisa, LatticeValue::Alisa) => LatticeValue::Alisa,
                (LatticeValue::Alisa, LatticeValue::Dead) => LatticeValue::Alisa,
                (LatticeValue::Dead, LatticeValue::Dead) => LatticeValue::Dead,
                (LatticeValue::Dead, LatticeValue::Alisa) => LatticeValue::Alisa,
                (_, _) => LatticeValue::StrongAlisa,
            },
            (Some(LatticeValue::Dead), None) => LatticeValue::Dead,
            (None, Some(LatticeValue::Dead)) => LatticeValue::Dead,
            (Some(LatticeValue::StrongAlisa), _) => LatticeValue::StrongAlisa,
            (_, Some(LatticeValue::StrongAlisa)) => LatticeValue::StrongAlisa,
            (_, _) => LatticeValue::Dead,
        }
    }

    /// Combine lattice value based on the instruction type and the facts we have had
    /// This is called in a transfer function on each instruction
    pub fn lattice_value_transfer(
        &self,
        instr: &Instruction,
        facts: &HashMap<String, LatticeValue>,
    ) -> HashMap<String, LatticeValue> {
        let mut sub_facts = HashMap::<String, LatticeValue>::new();

        if instr.is_nonlinear()
            || facts.get(&instr.dest.clone().unwrap_or("||||||||".to_string()))
                == Some(&LatticeValue::StrongAlisa)
        {
            // All args are now strongly live
            if let Some(args) = &instr.args {
                for arg in args {
                    sub_facts.insert(
                        arg.clone(),
                        self.lattice_value_meet(
                            Some(&LatticeValue::StrongAlisa),
                            facts.get(&arg.clone()),
                        ),
                    );
                }
            }
        } else {
            // All args are now live
            if let Some(args) = &instr.args {
                for arg in args {
                    sub_facts.insert(
                        arg.clone(),
                        self.lattice_value_meet(
                            Some(&LatticeValue::Alisa),
                            facts.get(&arg.clone()),
                        ),
                    );
                }
            }
        }
        let dead = match instr.is_nonlinear()
            || facts.get(&instr.dest.clone().unwrap_or("||||||||".to_string()))
                == Some(&LatticeValue::StrongAlisa)
        {
            true => LatticeValue::StrongAlisa,
            false => LatticeValue::Dead,
        };
        if let Some(dest) = &instr.dest {
            sub_facts.insert(
                dest.clone(),
                self.lattice_value_meet(
                    Some(&dead),
                    Some(&self.lattice_value_meet(
                        sub_facts.get(&dest.clone()),
                        facts.get(&dest.clone()),
                    )),
                ),
            );
        }
        sub_facts
    }
}
impl DataFlowAnalysis for LivenessAnalysis {
    /// Meet all the successor block based on the instruction's dest and LatticeValue
    fn meet(&mut self, bb: &mut BasicBlock) {
        let mut hs = HashMap::<String, LatticeValue>::new();

        //  In all predecessors's facts, we union them via the rule of
        //  combine_lattice_value
        for _ in bb.predecessors.iter() {
            for l in self.facts.entry(bb.id).or_default().clone() {
                let v = hs.get(&l.0);

                let res = self.lattice_value_meet(v, Some(&l.1));
                // eprintln!("Meet of {:?} in {:?}: meet value: {:?}", l.0, bb.instrs.first(), res);
                hs.insert(l.0, res);
            }
        }

        let v = self.facts.entry(bb.id).or_default();
        v.extend(hs.clone());
    }

    /// Transfer the facts in the block forwards
    fn transfer(&mut self, bb: &mut BasicBlock) -> TransferResult {
        let initial = self.facts.entry(bb.id).or_default().clone();

        // eprintln!("Transferring in {:?}", bb.instrs.first());
        for instr_label in bb.instrs.clone() {
            if let InstructionOrLabel::Instruction(instr) = instr_label {
                let sub_facts = self.lattice_value_transfer(&instr, &self.facts[&bb.id]);
                // eprintln!("Transferring in {:?}: {:?} - {:?}", bb.instrs.first(), a, b);
                self.facts.entry(bb.id).or_default().extend(sub_facts);
            }
        }

        let result = match initial == *self.facts.entry(bb.id).or_default() {
            true => TransferResult::NonChanged,
            false => TransferResult::Changed,
        };

        //eprintln!("{:?} in {:?}", result, bb.instrs.first());
        result
    }

    /// Transform a basic block based on the fact it has acquired, this is only after fix-point
    fn transform(&mut self, bb: &mut BasicBlock) {
        let mut keep = LinkedList::<InstructionOrLabel>::new();
        for instr_label in bb.instrs.iter_mut() {
            if let InstructionOrLabel::Instruction(instr) = instr_label {
                if instr.is_nonlinear() {
                    keep.push_back(InstructionOrLabel::from(instr.clone()));
                } else if let Some(LatticeValue::Dead) = self
                    .facts
                    .entry(bb.id)
                    .or_default()
                    .get(&instr.dest.clone().unwrap_or_else(|| "|||||||".to_string()))
                {
                } else {
                    keep.push_back(InstructionOrLabel::from(instr.clone()));
                }
            } else {
                keep.push_back(instr_label.clone());
            }
        }
        bb.instrs = keep;
    }

    fn get_dataflow_direction(&self) -> DataFlowDirection {
        DataFlowDirection::Backward
    }

    fn get_dataflow_order(&self) -> DataFlowOrder {
        DataFlowOrder::BFS
    }
}
fn main() {
    let mut prog = Program::stdin();

    let cfg = CFG::from_program(&mut prog);
    let mut d = LivenessAnalysis::new();
    cfg.dataflow(&mut d);

    let out_prog = cfg.to_program();
    out_prog.stdout()
}
