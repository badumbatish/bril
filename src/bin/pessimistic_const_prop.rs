use std::collections::{HashMap, HashSet};

use bril::bril_syntax::{BrilType, Instruction, InstructionOrLabel, Program};
use bril::util::{BasicBlock, TransferResult, CFG};
#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
pub enum LatticeValue {
    Dominator,
    Constant(u64),
}

pub fn combine_lattice_value(q: Option<&LatticeValue>, p: Option<&LatticeValue>) -> LatticeValue {
    match (q, p) {
        (Some(a), Some(b)) => match (a, b) {
            (LatticeValue::Dominator, LatticeValue::Dominator) => LatticeValue::Dominator,
            (LatticeValue::Dominator, LatticeValue::Constant(_)) => LatticeValue::Dominator,
            (LatticeValue::Constant(_), LatticeValue::Dominator) => LatticeValue::Dominator,
            (LatticeValue::Constant(c), LatticeValue::Constant(d)) => {
                if c == d {
                    LatticeValue::Constant(c.clone())
                } else {
                    LatticeValue::Dominator
                }
            }
        },
        (_, None) => LatticeValue::Dominator,
        (None, _) => LatticeValue::Dominator,
    }
}

/// Meet all the successor block
pub fn forward_meet(bb: &mut BasicBlock<LatticeValue>) {
    let mut hs = HashMap::<String, LatticeValue>::new();

    for i in bb.predecessors.iter() {
        for l in i.borrow_mut().facts.clone() {
            let v = hs.get(&l.0);
            let res = combine_lattice_value(v, Some(&l.1));

            hs.insert(l.0, res);
        }
    }

    bb.facts = hs;
}

pub fn is_const(instr: Instruction, facts: &HashMap<String, LatticeValue>) -> LatticeValue {
    if instr.is_add() {
        let args = instr.args.clone().unwrap();
        let a = facts.get(&args[0]);
        let b = facts.get(&args[1]);

        return match (a, b) {
            (Some(LatticeValue::Constant(c)), Some(LatticeValue::Constant(d))) => {
                LatticeValue::Constant(c + d)
            }
            _ => LatticeValue::Dominator,
        };
    }

    LatticeValue::Dominator
}
/// Transfer the facts in the block forwards
pub fn forward_transfer(bb: &mut BasicBlock<LatticeValue>) -> TransferResult {
    let mut changed = TransferResult::NonChanged;
    let initial = bb.facts.clone();
    for instr_label in bb.instrs.clone() {
        match instr_label {
            InstructionOrLabel::Instruction(instr) => {
                // if instr.is_const() && instr.bril_type == Some(BrilType::Int) {
                //     let lv = bb.facts.get(&instr.dest.clone().unwrap().clone());
                //     let a = match lv {
                //         Some(l) => Some(l.clone()),
                //         None => Some(LatticeValue::Dominator),
                //     };
                //     let change = bb.facts.insert(
                //         instr.clone().dest.clone().expect("Should not happen"),
                //         combine_lattice_value(
                //             a.as_ref(),
                //             Some(&LatticeValue::Constant(
                //                 serde_json::from_value(instr.value.unwrap())
                //                     .expect("This cant fail"),
                //             )),
                //         ),
                //     );
                //
                //     if change.is_none() {
                //         changed = TransferResult::CHANGED;
                //     } else if a != change {
                //         changed = TransferResult::CHANGED;
                //     }
                // } else if let LatticeValue::Constant(c) = is_const(instr, &bb.facts) {
                // }
            }
            InstructionOrLabel::Label(_) => continue,
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
        match instr_label {
            InstructionOrLabel::Instruction(instr) => {
                // if instr.dest.is_some() {
                //     let q = bb.facts.get(&instr.dest.clone().unwrap());
                //     if let Some(LatticeValue::Constant(b)) = q {
                //         instr.to_const_int(*b);
                //         instr.args = None;
                //         instr.funcs = None;
                //     }
                // }
            }

            _ => continue,
        }
    }
}

fn main() {
    let prog = Program::stdin();

    let mut cfg = CFG::from_program(prog);

    cfg.dataflow_forward(forward_meet, forward_transfer, transform);
    let prog = cfg.to_program();

    prog.stdout()
    // cfg.print_hm();
}
