use std::collections::HashMap;

use bril::bril_syntax::{BrilType, Instruction, InstructionOrLabel, Program};
use bril::cfg::{BasicBlock, ConditionalDataFlowAnalysis, ConditionalTransferResult, CFG};
#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
pub enum LatticeValue {
    Dominator,
    Dominated,
    ConstantInt(i64),
    ConstantBool(bool),
}

pub struct OptimisticConstProp {
    pub facts: HashMap<usize, HashMap<String, LatticeValue>>,
}
impl Default for OptimisticConstProp {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimisticConstProp {
    pub fn new() -> Self {
        Self {
            facts: HashMap::default(),
        }
    }
    /// Combine lattice value based on the lattice value type
    /// This is called in a meet function on each instruction
    pub fn lattice_value_meet(
        &self,
        q: Option<&LatticeValue>,
        p: Option<&LatticeValue>,
    ) -> LatticeValue {
        
        //eprintln!("{:?} : {:?} : {:?}", meet_value, q, p);
        match (q, p) {
            (Some(a), Some(b)) => match (a, b) {
                (LatticeValue::Dominator, LatticeValue::Dominator) => LatticeValue::Dominator,
                (LatticeValue::Dominator, LatticeValue::ConstantInt(_)) => LatticeValue::Dominator,
                (LatticeValue::Dominator, LatticeValue::ConstantBool(_)) => LatticeValue::Dominator,
                (LatticeValue::Dominator, LatticeValue::Dominated) => LatticeValue::Dominator,
                (LatticeValue::ConstantInt(_), LatticeValue::Dominator) => LatticeValue::Dominator,
                (LatticeValue::ConstantInt(_), LatticeValue::ConstantBool(_)) => {
                    LatticeValue::Dominator
                }
                (LatticeValue::ConstantInt(c), LatticeValue::Dominated) => {
                    LatticeValue::ConstantInt(*c)
                }
                (LatticeValue::ConstantInt(c), LatticeValue::ConstantInt(d)) => {
                    if c == d {
                        LatticeValue::ConstantInt(*c)
                    } else {
                        LatticeValue::Dominator
                    }
                }
                (LatticeValue::ConstantBool(_), LatticeValue::Dominator) => LatticeValue::Dominator,
                (LatticeValue::ConstantBool(c), LatticeValue::Dominated) => {
                    LatticeValue::ConstantBool(*c)
                }
                (LatticeValue::ConstantBool(c), LatticeValue::ConstantBool(d)) => {
                    if c == d {
                        LatticeValue::ConstantBool(*c)
                    } else {
                        LatticeValue::Dominator
                    }
                }
                (LatticeValue::ConstantBool(_), LatticeValue::ConstantInt(_)) => {
                    LatticeValue::Dominator
                }
                (LatticeValue::Dominated, LatticeValue::Dominator) => LatticeValue::Dominator,
                (LatticeValue::Dominated, LatticeValue::Dominated) => LatticeValue::Dominated,
                (LatticeValue::Dominated, LatticeValue::ConstantInt(c)) => {
                    LatticeValue::ConstantInt(*c)
                }
                (LatticeValue::Dominated, LatticeValue::ConstantBool(c)) => {
                    LatticeValue::ConstantBool(*c)
                }
            },
            (_, Some(LatticeValue::ConstantInt(c))) => LatticeValue::ConstantInt(*c),
            (Some(LatticeValue::ConstantInt(c)), _) => LatticeValue::ConstantInt(*c),
            (_, Some(LatticeValue::ConstantBool(c))) => LatticeValue::ConstantBool(*c),
            (Some(LatticeValue::ConstantBool(c)), _) => LatticeValue::ConstantBool(*c),
            (Some(LatticeValue::Dominated), _) => LatticeValue::Dominated,
            (_, Some(LatticeValue::Dominated)) => LatticeValue::Dominated,
            (None, None) => LatticeValue::Dominated,
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
            if instr.bril_type.is_some() && *instr.bril_type.as_ref().unwrap() == BrilType::Int {
                Some((
                    instr.clone().dest?,
                    LatticeValue::ConstantInt(
                        (serde_json::from_value(instr.value.clone().unwrap()))
                            .expect("Failed to parse value "),
                    ),
                ))
            } else if instr.bril_type.is_some()
                && *instr.bril_type.as_ref().unwrap() == BrilType::Bool
            {
                Some((
                    instr.clone().dest?,
                    LatticeValue::ConstantBool(
                        (serde_json::from_value(instr.value.clone().unwrap()))
                            .expect("Failed to parse value "),
                    ),
                ))
            } else {
                None
            }
        } else if instr.is_id() {
            match facts.get(&instr.args.clone().unwrap()[0]) {
                Some(LatticeValue::ConstantInt(c)) => {
                    Some((instr.clone().dest?, LatticeValue::ConstantInt(*c)))
                }
                Some(LatticeValue::ConstantBool(c)) => {
                    Some((instr.clone().dest?, LatticeValue::ConstantBool(*c)))
                }
                _ => None,
            }
        } else if instr.is_add() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::ConstantInt(c)), Some(LatticeValue::ConstantInt(d))) => {
                    Some((instr.clone().dest?, LatticeValue::ConstantInt(c + d)))
                }
                _ => None,
            }
        } else if instr.is_sub() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::ConstantInt(c)), Some(LatticeValue::ConstantInt(d))) => {
                    Some((instr.clone().dest?, LatticeValue::ConstantInt(c - d)))
                }
                _ => None,
            }
        } else if instr.is_mul() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::ConstantInt(c)), Some(LatticeValue::ConstantInt(d))) => {
                    Some((instr.clone().dest?, LatticeValue::ConstantInt(c * d)))
                }
                _ => None,
            }
        } else if instr.is_div() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::ConstantInt(c)), Some(LatticeValue::ConstantInt(d))) => {
                    Some((instr.clone().dest?, LatticeValue::ConstantInt(c / d)))
                }
                _ => None,
            }
        } else if instr.is_le() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::ConstantInt(c)), Some(LatticeValue::ConstantInt(d))) => {
                    Some((instr.clone().dest?, LatticeValue::ConstantBool(c <= d)))
                }
                _ => None,
            }
        } else if instr.is_ge() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::ConstantInt(c)), Some(LatticeValue::ConstantInt(d))) => {
                    Some((instr.clone().dest?, LatticeValue::ConstantBool(c >= d)))
                }
                _ => None,
            }
        } else if instr.is_lt() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::ConstantInt(c)), Some(LatticeValue::ConstantInt(d))) => {
                    Some((instr.clone().dest?, LatticeValue::ConstantBool(c < d)))
                }
                _ => None,
            }
        } else if instr.is_gt() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::ConstantInt(c)), Some(LatticeValue::ConstantInt(d))) => {
                    Some((instr.clone().dest?, LatticeValue::ConstantBool(c > d)))
                }
                _ => None,
            }
        } else if instr.is_eq() {
            let args = instr.args.clone().unwrap();
            let a = facts.get(&args[0]);
            let b = facts.get(&args[1]);

            match (a, b) {
                (Some(LatticeValue::ConstantInt(c)), Some(LatticeValue::ConstantInt(d))) => {
                    Some((instr.clone().dest?, LatticeValue::ConstantBool(c == d)))
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
impl ConditionalDataFlowAnalysis for OptimisticConstProp {
    /// Meet all the successor block based on the instruction's dest and LatticeValue
    fn meet(&mut self, bb: &mut BasicBlock) {
        let mut hs = HashMap::<String, LatticeValue>::new();
        //eprintln!("Hello in {:?}", bb.instrs.first());
        //  In all predecessors's facts, we union them via the rule of
        //  combine_lattice_value
        for i in bb.predecessors.iter() {
            for l in self.facts.entry(i.borrow().id).or_default().clone() {
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
    fn transfer(&mut self, bb: &mut BasicBlock) -> ConditionalTransferResult {
        let initial = self.facts.entry(bb.id).or_default().clone();
        //eprintln!("Transferring in {:?}", bb.instrs.first());
        for instr_label in bb.instrs.clone() {
            if let InstructionOrLabel::Instruction(instr) = instr_label {
                if let Some((a, b)) = self.lattice_value_transfer(&instr, &self.facts[&bb.id]) {
                    //eprintln!("Transferring in {:?}: {:?} - {:?}", bb.instrs.first(), a, b);
                    let v = self.facts.entry(bb.id).or_default();
                    v.insert(a, b);
                }
            }
        }

        let result = match bb.instrs.last() {
            Some(instr_lb) => match instr_lb {
                InstructionOrLabel::Label(_) => match initial == self.facts[&bb.id] {
                    true => ConditionalTransferResult::NoPathTaken,
                    false => ConditionalTransferResult::AllPathTaken,
                },

                InstructionOrLabel::Instruction(instruction) => {
                    if instruction.is_br() {
                        match self.facts[&bb.id].get(&instruction.args.clone().unwrap().clone()[0])
                        {
                            Some(LatticeValue::ConstantBool(true)) => {
                                ConditionalTransferResult::FirstPathTaken
                            }
                            Some(LatticeValue::ConstantBool(false)) => {
                                ConditionalTransferResult::SecondPathTaken
                            }
                            _ => match initial == self.facts[&bb.id] {
                                true => ConditionalTransferResult::NoPathTaken,
                                false => ConditionalTransferResult::AllPathTaken,
                            },
                        }
                    } else {
                        match initial == self.facts[&bb.id] {
                            true => ConditionalTransferResult::NoPathTaken,
                            false => ConditionalTransferResult::AllPathTaken,
                        }
                    }
                }
            },
            None => panic!("Logically cannot happen"),
        };

        //eprintln!("Result {:?}", result);
        result
    }

    /// Transform a basic block based on the fact it has acquired, this is only after fix-point
    fn transform(&mut self, bb: &mut BasicBlock) {
        for instr_label in bb.instrs.iter_mut() {
            if let InstructionOrLabel::Instruction(instr) = instr_label {
                if instr.is_nonlinear() {
                    continue;
                }
                if instr.dest.is_some() {
                    if let Some(LatticeValue::ConstantInt(c)) = &self
                        .facts
                        .entry(bb.id)
                        .or_default()
                        .get(&instr.dest.clone().unwrap())
                    {
                        instr.value =
                            Some(serde_json::to_value(c).expect("This should absolutely not fail"));
                        instr.args = None;
                        instr.op = "const".to_string();
                        instr.funcs = None;
                    } else if let Some(LatticeValue::ConstantBool(c)) =
                        &self.facts[&bb.id].get(&instr.dest.clone().unwrap())
                    {
                        instr.value =
                            Some(serde_json::to_value(c).expect("This should absolutely not fail"));
                        instr.args = None;
                        instr.op = "const".to_string();
                        instr.funcs = None;
                    }
                }
            }
        }
    }
}
fn main() {
    let prog = Program::stdin();

    let cfg = CFG::from_program(prog);
    let mut d = OptimisticConstProp::new();
    cfg.dataflow_forward_optimistically(&mut d);
    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
