use std::collections::HashMap;

use bril::bril_syntax::{Instruction, InstructionOrLabel, Program};
use bril::cfg::{BasicBlock, TransferResult, CFG};
#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
pub enum LatticeValue {
    Alisa,
    Dead,
    StrongAlisa,
}

/// Combine lattice value based on the lattice value type
/// This is called in a meet function on each instruction
pub fn lattice_value_meet(q: Option<&LatticeValue>, p: Option<&LatticeValue>) -> LatticeValue {
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
        (_, _) => LatticeValue::Alisa,
    }
}

/// Combine lattice value based on the instruction type and the facts we have had
/// This is called in a transfer function on each instruction
pub fn lattice_value_transfer(
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
                    lattice_value_meet(Some(&LatticeValue::StrongAlisa), facts.get(&arg.clone())),
                );
            }
        }
    } else {
        // All args are now live
        if let Some(args) = &instr.args {
            for arg in args {
                sub_facts.insert(
                    arg.clone(),
                    lattice_value_meet(Some(&LatticeValue::Alisa), facts.get(&arg.clone())),
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
            lattice_value_meet(
                Some(&dead),
                Some(&lattice_value_meet(
                    sub_facts.get(&dest.clone()),
                    facts.get(&dest.clone()),
                )),
            ),
        );
    }
    sub_facts
}
/// Meet all the successor block based on the instruction's dest and LatticeValue
pub fn backward_meet(bb: &mut BasicBlock<LatticeValue>) {
    let mut hs = HashMap::<String, LatticeValue>::new();

    //  In all predecessors's facts, we union them via the rule of
    //  combine_lattice_value
    for i in bb.predecessors.iter() {
        for l in i.borrow_mut().facts.clone() {
            let v = hs.get(&l.0);

            let res = lattice_value_meet(v, Some(&l.1));
            // eprintln!("Meet of {:?} in {:?}: meet value: {:?}", l.0, bb.instrs.first(), res);
            hs.insert(l.0, res);
        }
    }

    bb.facts = hs;
}

/// Transfer the facts in the block forwards
pub fn backward_transfer(bb: &mut BasicBlock<LatticeValue>) -> TransferResult {
    let initial = bb.facts.clone();
    // eprintln!("Transferring in {:?}", bb.instrs.first());
    for instr_label in bb.instrs.clone() {
        if let InstructionOrLabel::Instruction(instr) = instr_label {
            let sub_facts = lattice_value_transfer(&instr, &bb.facts);
            // eprintln!("Transferring in {:?}: {:?} - {:?}", bb.instrs.first(), a, b);
            bb.facts.extend(sub_facts);
        }
    }

    let result = match initial == bb.facts {
        true => TransferResult::NonChanged,
        false => TransferResult::Changed,
    };

    //eprintln!("{:?} in {:?}", result, bb.instrs.first());
    result
}

/// Transform a basic block based on the fact it has acquired, this is only after fix-point
pub fn transform(bb: &mut BasicBlock<LatticeValue>) {
    let mut keep = Vec::<InstructionOrLabel>::new();
    for instr_label in bb.instrs.iter_mut() {
        if let InstructionOrLabel::Instruction(instr) = instr_label {
            if instr.is_nonlinear() {
                keep.push(InstructionOrLabel::from(instr.clone()));
            } else if let Some(LatticeValue::Dead) = &bb
                .facts
                .get(&instr.dest.clone().unwrap_or_else(|| "|||||||".to_string()))
            {
            } else {
                keep.push(InstructionOrLabel::from(instr.clone()));
            }
        } else {
            keep.push(instr_label.clone());
        }
    }
    bb.instrs = keep;
}

fn main() {
    let prog = Program::stdin();

    let cfg = CFG::from_program(prog);

    cfg.dataflow_backward(backward_meet, backward_transfer, transform);
    let prog = cfg.to_program();

    prog.stdout()
}
