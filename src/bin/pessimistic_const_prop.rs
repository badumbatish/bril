use std::collections::HashMap;

use bril::basic_block::BasicBlock;
use bril::bril_syntax::{Instruction, InstructionOrLabel, Program};
use bril::cfg::{DataFlowAnalysis, DataFlowDirection, TransferResult, CFG};
#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
pub enum LatticeValue {
    Dominator,
    Constant(i64),
}

pub struct PessimisticConstProp {
    pub facts: HashMap<usize, HashMap<String, LatticeValue>>,
}
impl Default for PessimisticConstProp {
    fn default() -> Self {
        Self::new()
    }
}

impl PessimisticConstProp {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
        }
    }
    pub fn lattice_value_meet(
        &self,
        q: Option<&LatticeValue>,
        p: Option<&LatticeValue>,
    ) -> LatticeValue {
        // eprintln!("{:?} : {:?} : {:?}", meet_value, q, p);
        match (q, p) {
            (Some(a), Some(b)) => match (a, b) {
                (LatticeValue::Dominator, LatticeValue::Dominator) => LatticeValue::Dominator,
                (LatticeValue::Dominator, LatticeValue::Constant(_)) => LatticeValue::Dominator,
                (LatticeValue::Constant(_), LatticeValue::Dominator) => LatticeValue::Dominator,
                (LatticeValue::Constant(c), LatticeValue::Constant(d)) => {
                    if c == d {
                        LatticeValue::Constant(*c)
                    } else {
                        LatticeValue::Dominator
                    }
                }
            },
            (_, _) => LatticeValue::Dominator,
        }
    }

    /// Combine lattice value based on the instruction type and the facts we have had
    /// This is called in a transfer function on each instruction
    pub fn lattice_value_transfer(
        &self,
        instr: &Instruction,
        facts: &HashMap<String, LatticeValue>,
    ) -> Option<(String, LatticeValue)> {
        let a = if instr.is_const() {
            Some((
                instr.clone().dest?,
                LatticeValue::Constant(
                    (serde_json::from_value(instr.value.clone().unwrap()))
                        .expect("Failed to parse value "),
                ),
            ))
        } else if instr.is_add() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::Constant(c)), Some(LatticeValue::Constant(d))) => {
                    Some((instr.clone().dest?, LatticeValue::Constant(c + d)))
                }
                _ => None,
            }
        } else if instr.is_sub() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::Constant(c)), Some(LatticeValue::Constant(d))) => {
                    Some((instr.clone().dest?, LatticeValue::Constant(c - d)))
                }
                _ => None,
            }
        } else if instr.is_mul() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::Constant(c)), Some(LatticeValue::Constant(d))) => {
                    Some((instr.clone().dest?, LatticeValue::Constant(c * d)))
                }
                _ => None,
            }
        } else if instr.is_div() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::Constant(c)), Some(LatticeValue::Constant(d))) => {
                    Some((instr.clone().dest?, LatticeValue::Constant(c / d)))
                }
                _ => None,
            }
        } else {
            None
        };

        match a {
            Some(fact) => match &instr.dest {
                Some(d) => Some((
                    (&d).to_string(),
                    self.lattice_value_meet(Some(&fact.1), facts.get(&d.clone())),
                )),
                _ => None,
            },
            _ => None,
        }
    }
}

impl DataFlowAnalysis for PessimisticConstProp {
    fn get_dataflow_direction(&self) -> bril::cfg::DataFlowDirection {
        DataFlowDirection::Forward
    }
    /// Meet all the successor block based on the instruction's dest and LatticeValue
    fn meet(&mut self, bb: &mut BasicBlock) {
        let mut hs = HashMap::<String, LatticeValue>::new();

        //  In all predecessors's facts, we union them via the rule of
        //  combine_lattice_value
        for i in bb.predecessors.iter() {
            for l in self.facts[&i.borrow().id].clone() {
                let v = hs.get(&l.0);
                //eprintln!("Meet of {:?} in {:?}", l.0, bb.instrs.first());
                let res = self.lattice_value_meet(v, Some(&l.1));

                hs.insert(l.0, res);
            }
        }
        let v = self.facts.entry(bb.id).or_default();
        v.extend(hs.clone());
    }

    /// Transfer the facts in the block forwards
    fn transfer(&mut self, bb: &mut BasicBlock) -> TransferResult {
        let initial = self.facts[&bb.id].clone();
        //eprintln!("Transferring in {:?}", bb.instrs.first());
        for instr_label in bb.instrs.clone() {
            if let InstructionOrLabel::Instruction(instr) = instr_label {
                if let Some((a, b)) = self.lattice_value_transfer(&instr, &self.facts[&bb.id]) {
                    // eprintln!("Transferring in {:?}: {:?} - {:?}", bb.instrs.first(), a, b);
                    let v = self.facts.entry(bb.id).or_default();
                    v.insert(a, b);
                }
            }
        }

        match initial == self.facts[&bb.id] {
            true => TransferResult::NonChanged,
            false => TransferResult::Changed,
        }
    }

    /// Transform a basic block based on the fact it has acquired, this is only after fix-point
    fn transform(&mut self, bb: &mut BasicBlock) {
        for instr_label in bb.instrs.iter_mut() {
            if let InstructionOrLabel::Instruction(instr) = instr_label {
                if instr.is_nonlinear() {
                    continue;
                }
                if let Some(LatticeValue::Constant(c)) = &self.facts[&bb.id]
                    .get(&instr.dest.clone().unwrap_or_else(|| "|||||||".to_string()))
                {
                    instr.value =
                        Some(serde_json::to_value(c).expect("This should absolutely not failed"));
                    instr.args = None;
                    instr.op = "const".to_string();
                    instr.funcs = None;
                }
            }
        }
    }

    fn get_dataflow_order(&self) -> bril::cfg::DataFlowOrder {
        bril::cfg::DataFlowOrder::BFS
    }
}

/// Combine lattice value based on the lattice value type
/// This is called in a meet function on each instruction

fn main() {
    let prog = Program::stdin();

    let cfg = CFG::from_program(prog);
    let mut pessi = PessimisticConstProp::new();
    cfg.dataflow(&mut pessi);
    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
