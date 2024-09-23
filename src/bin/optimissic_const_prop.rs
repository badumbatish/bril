use std::collections::HashMap;

use bril::bril_syntax::{Instruction, InstructionOrLabel, Program};
use bril::util::{BasicBlock, TransferResult, CFG};
#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
pub enum LatticeValue {
    Dominator,
    Dominated,
    Constant(u64),
}

/// Combine lattice value based on the lattice value type
/// This is called in a meet function on each instruction
pub fn lattice_value_meet(q: Option<&LatticeValue>, p: Option<&LatticeValue>) -> LatticeValue {
    let meet_value = match (q, p) {
        (Some(a), Some(b)) => match (a, b) {
            (LatticeValue::Dominator, LatticeValue::Dominator) => LatticeValue::Dominator,
            (LatticeValue::Dominator, LatticeValue::Constant(_)) => LatticeValue::Dominator,
            (LatticeValue::Dominator, LatticeValue::Dominated) => LatticeValue::Dominator,
            (LatticeValue::Constant(_), LatticeValue::Dominator) => LatticeValue::Dominator,
            (LatticeValue::Constant(c), LatticeValue::Dominated) => LatticeValue::Constant(*c),
            (LatticeValue::Constant(c), LatticeValue::Constant(d)) => {
                if c == d {
                    LatticeValue::Constant(*c)
                } else {
                    LatticeValue::Dominator
                }
            }
            (LatticeValue::Dominated, LatticeValue::Dominator) => LatticeValue::Dominator,
            (LatticeValue::Dominated, LatticeValue::Dominated) => LatticeValue::Dominated,
            (LatticeValue::Dominated, LatticeValue::Constant(c)) => LatticeValue::Constant(*c),
        },
        (_, Some(LatticeValue::Constant(c))) => LatticeValue::Constant(*c),
        (Some(LatticeValue::Constant(c)), _) => LatticeValue::Constant(*c),
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
pub fn forward_transfer(bb: &mut BasicBlock<LatticeValue>) -> TransferResult {
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

    match initial == bb.facts {
        true => TransferResult::NonChanged,
        false => TransferResult::CHANGED,
    }
}

/// Transform a basic block based on the fact it has acquired, this is only after fix-point
pub fn transform(bb: &mut BasicBlock<LatticeValue>) {
    for instr_label in bb.instrs.iter_mut() {
        if let InstructionOrLabel::Instruction(instr) = instr_label {
            if instr.is_nonlinear() {
                continue;
            }
            if let Some(LatticeValue::Constant(c)) = &bb
                .facts
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

fn main() {
    let prog = Program::stdin();

    let cfg = CFG::from_program(prog);

    cfg.dataflow_forward(forward_meet, forward_transfer, transform);
    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
