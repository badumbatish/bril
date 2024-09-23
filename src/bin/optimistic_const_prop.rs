use std::collections::HashMap;

use bril::bril_syntax::{BrilType, Instruction, InstructionOrLabel, Program};
use bril::util::{BasicBlock, ConditionalTransferResult, TransferResult, CFG};
#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
pub enum LatticeValue {
    Dominator,
    Dominated,
    ConstantInt(u64),
    ConstantBool(bool),
}
const DEAD_CODE: &str = "|DEAD_CODE|";
/// Combine lattice value based on the lattice value type
/// This is called in a meet function on each instruction
pub fn lattice_value_meet(q: Option<&LatticeValue>, p: Option<&LatticeValue>) -> LatticeValue {
    let meet_value = match (q, p) {
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
            (LatticeValue::ConstantBool(c), LatticeValue::ConstantInt(d)) => {
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
    };
    eprintln!("{:?} : {:?} : {:?}", meet_value, q, p);
    meet_value
}

/// Combine lattice value based on the instruction type and the facts we have had
/// This is called in a transfer function on each instruction
pub fn lattice_value_transfer(
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
        } else {
            Some((
                instr.clone().dest?,
                LatticeValue::ConstantBool(
                    (serde_json::from_value(instr.value.clone().unwrap()))
                        .expect("Failed to parse value "),
                ),
            ))
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
                lattice_value_meet(Some(&fact.1), facts.get(&d.clone())),
            )),
            _ => None,
        },
        _ => None,
    }
}
/// Meet all the successor block based on the instruction's dest and LatticeValue
pub fn forward_meet(bb: &mut BasicBlock<LatticeValue>) {
    let mut hs = HashMap::<String, LatticeValue>::new();

    //  In all predecessors's facts, we union them via the rule of
    //  combine_lattice_value
    for i in bb.predecessors.iter() {
        for l in i.borrow_mut().facts.clone() {
            let v = hs.get(&l.0);
            eprintln!("Meet of {:?} in {:?}", l.0, bb.instrs.first());
            let res = lattice_value_meet(v, Some(&l.1));

            hs.insert(l.0, res);
        }
    }

    bb.facts = hs;
}

/// Transfer the facts in the block forwards
pub fn forward_transfer(bb: &mut BasicBlock<LatticeValue>) -> ConditionalTransferResult {
    let initial = bb.facts.clone();
    eprintln!("Transferring in {:?}", bb.instrs.first());
    for instr_label in bb.instrs.clone() {
        if let InstructionOrLabel::Instruction(instr) = instr_label {
            if let Some((a, b)) = lattice_value_transfer(&instr, &bb.facts) {
                eprintln!("Transferring in {:?}: {:?} - {:?}", bb.instrs.first(), a, b);
                bb.facts.insert(a, b);
            }
        }
    }

    match bb.instrs.last() {
        Some(instr_lb) => match instr_lb {
            InstructionOrLabel::Label(label) => ConditionalTransferResult::AllPathTaken,
            InstructionOrLabel::Instruction(instruction) => {
                if instruction.is_br() {
                    match bb.facts.get(&instruction.args.clone().unwrap().clone()[0]) {
                        Some(LatticeValue::ConstantBool(true)) => {
                            ConditionalTransferResult::FirstPathTaken
                        }
                        Some(LatticeValue::ConstantBool(false)) => {
                            ConditionalTransferResult::SecondPathTaken
                        }
                        _ => ConditionalTransferResult::AllPathTaken,
                    }
                } else {
                    match initial == bb.facts {
                        true => ConditionalTransferResult::NoPathTaken,
                        false => ConditionalTransferResult::AllPathTaken,
                    }
                }
            }
        },
        None => panic!("Logically cannot happen"),
    }
}

/// Transform a basic block based on the fact it has acquired, this is only after fix-point
pub fn transform(bb: &mut BasicBlock<LatticeValue>) {
    for instr_label in bb.instrs.iter_mut() {
        if let InstructionOrLabel::Instruction(instr) = instr_label {
            if instr.is_nonlinear() {
                continue;
            }
            if instr.dest.is_some() {
                if let Some(LatticeValue::ConstantInt(c)) =
                    &bb.facts.get(&instr.dest.clone().unwrap())
                {
                    instr.value =
                        Some(serde_json::to_value(c).expect("This should absolutely not failed"));
                    instr.args = None;
                    instr.op = "const".to_string();
                    instr.funcs = None;
                } else if let Some(LatticeValue::ConstantBool(c)) =
                    &bb.facts.get(&instr.dest.clone().unwrap())
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
}

fn main() {
    let prog = Program::stdin();

    let cfg = CFG::from_program(prog);

    cfg.dataflow_forward_optimistically(forward_meet, forward_transfer, transform);
    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
