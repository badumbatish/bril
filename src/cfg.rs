use crate::aliases::{BbPtr, BlockID, IdToBbMap};
use crate::basic_block::BasicBlock;
use crate::bril_syntax::{Function, InstructionOrLabel, Program};
use crate::dominance::DominanceDataFlow;
use crate::loops::Loops;
use std::collections::{LinkedList, VecDeque};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt::Debug,
};

#[derive(Debug)]
pub struct CFG {
    pub hm: HashMap<InstructionOrLabel, BbPtr>,
    pub basic_block_counter: BlockID,
    pub instruction_counter: usize,
    pub id_to_bb: IdToBbMap,
    pub bb_ptr_vec: LinkedList<BbPtr>,
}
// main:
// @main
impl CFG {
    //pub fn components_from_function(
    //    f: Function,
    //) -> (
    //    HashMap<InstructionOrLabel, BbPtr>,
    //    HashMap<BlockID, BbPtr>,
    //) {
    //    // O(n)
    //    let mut hm = HashMap::<InstructionOrLabel, BbPtr>::new();
    //    let mut id_to_bb = HashMap::<BlockID, BbPtr>::new();
    //    let mut basic_block_counter = 0;
    //    let simple_basic_blocks_vec_from_function =
    //        BasicBlock::simple_basic_blocks_vec_from_function(f, &mut basic_block_counter);
    //
    //    for bb in simple_basic_blocks_vec_from_function {
    //        let bb_clone = bb.clone();
    //        hm.insert(
    //            bb_clone.borrow().instrs.front().unwrap().clone(),
    //            bb.clone(),
    //        );
    //        id_to_bb.insert(bb.borrow().id, bb.clone());
    //    }
    //
    //    (hm, id_to_bb)
    //    // O(n)
    //}
    pub fn from_program(p: &mut Program) -> Self {
        let mut hm = HashMap::<InstructionOrLabel, BbPtr>::new();

        let mut id_to_bb = HashMap::<BlockID, BbPtr>::new();

        let mut bb_ptr_vec = LinkedList::<BbPtr>::new();
        let mut basic_block_counter: BlockID = 0;
        let mut instruction_counter: usize = 0;
        // Initialize the id first.
        for func in p.functions.iter_mut() {
            for instr in func.instrs.iter_mut() {
                match instr {
                    InstructionOrLabel::Instruction(ref mut i) => {
                        //eprintln!("Adding instruction counter {instruction_counter}");
                        i.instruction_id = Some(instruction_counter);
                        instruction_counter += 1;
                    }
                    InstructionOrLabel::Label(_l) => {}
                }
            }
        }
        // Iterate to put basic blocks into the graph
        for func in p.functions.iter() {
            let simple_basic_blocks_vec_from_function =
                BasicBlock::simple_basic_blocks_vec_from_function(func, &mut basic_block_counter);
            for bb in simple_basic_blocks_vec_from_function {
                hm.insert(bb.borrow().instrs.front().unwrap().clone(), bb.clone());
                id_to_bb.insert(bb.borrow().id, bb.clone());
                bb_ptr_vec.push_back(bb.clone());
            }
        }
        // Iterate to connect them
        for i in hm.clone().into_values() {
            let mut bi = i.borrow_mut();
            if !bi.instrs.is_empty() {
                match bi.instrs.clone().into_iter().last() {
                    Some(instr) => match instr {
                        InstructionOrLabel::Label(_) => {
                            //eprintln!("This should not happen in CFG::from_program")
                        }
                        InstructionOrLabel::Instruction(ins) => {
                            if ins.is_br() {
                                let first_branch_label = &InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[1].clone(),
                                );
                                let second_branch_label = &InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[0].clone(),
                                );
                                bi.successors.push(hm[first_branch_label].clone());
                                hm[first_branch_label]
                                    .borrow_mut()
                                    .predecessors
                                    .push(i.clone());

                                bi.successors.push(hm[second_branch_label].clone());
                                hm[second_branch_label]
                                    .borrow_mut()
                                    .predecessors
                                    .push(i.clone());
                            } else if ins.is_jmp() {
                                let first_branch_label = &InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[0].clone(),
                                );
                                bi.successors.push(hm[first_branch_label].clone());
                                hm[first_branch_label]
                                    .borrow_mut()
                                    .predecessors
                                    .push(i.clone());
                            }
                        }
                    },
                    None => continue,
                }
            }
        }

        Self {
            hm,
            basic_block_counter,
            instruction_counter,
            id_to_bb,
            bb_ptr_vec,
        }
    }

    pub fn to_program(&self) -> Program {
        let mut p = Program {
            functions: Vec::default(),
            // other_fields: serde_json::Value::default(),
        };

        // The function is here just because we want to maintain the initial order of function in
        // textual IR
        // If encountered a function,
        #[warn(unused_assignments)]
        let dummy_func: Function = Function {
            name: "Dummy".to_string(),
            instrs: Vec::new(),
            args: None,
            bril_type: None,
        };

        let mut func = dummy_func.clone();
        let mut first_func = true;

        //eprintln!("Size of bb ptr vec : {}", self.bb_ptr_vec.len());
        for bb_ptr in self.bb_ptr_vec.iter() {
            //eprintln!("{}", bb_ptr.borrow().func.is_some());
            if bb_ptr.borrow().func.is_some() {
                if first_func {
                    first_func = false;
                } else {
                    p.functions.push(func);
                }
                func = bb_ptr.borrow().func.clone().unwrap();
                func.instrs.clear();
            }
            //eprintln!("Getting instr from {}", bb_ptr.borrow().id);
            for instr in bb_ptr.borrow().instrs.iter() {
                func.instrs.push(instr.clone());
                match func.instrs.last_mut().unwrap() {
                    InstructionOrLabel::Label(_) => {}
                    InstructionOrLabel::Instruction(instruction) => {
                        if instruction.instruction_id.is_none() {
                            panic!("Goddammnit, I can't afford to have a null instruction id, each need to be accounted for so that I can perform alias analysis")
                        }
                    }
                }
            }
        }
        p.functions.push(func);
        p
    }

    pub fn print_hm(&self) {
        for i in self.hm.iter() {
            eprintln!("{:?}", i.0);
            eprintln!("{:?}", i.1.borrow())
        }
    }

    //pub fn to_dot_string(&self) -> String {
    //    let mut graph_as_string = String::from("digraph {\n");
    //
    //    for i in self.hm.iter() {
    //        let node = format!(
    //            "{} [label=\"{} \"] \n",
    //            Self::make_node_name(*i.0),
    //            Self::make_info_from_block(i.1.clone())
    //        );
    //        graph_as_string.push_str(&node);
    //    }
    //
    //    for i in self.hm.iter() {
    //        graph_as_string += &Self::make_rel_from_block(i.1.clone())
    //    }
    //    // Setting up edge for the nodes
    //
    //    graph_as_string.push_str("}");
    //
    //    graph_as_string
    //}
    //fn make_node_name(id: u32) -> String {
    //    "\tnode".to_owned() + &id.to_string()
    //}
    //fn make_info_from_block(bb: BbPtr) -> String {
    //    let mut result = String::new();
    //
    //    match &bb.borrow().leader {
    //        Leader::FunctionName(fn_name) => {
    //            result.push_str(&("@".to_owned() + &fn_name.clone() + "(...) \\n"));
    //            for i in bb.borrow().instrs.iter().skip(1) {
    //                result += &(i.to_string() + "\\n");
    //            }
    //        }
    //        Leader::InstructionOrLabel(_) => {
    //            for i in bb.borrow().instrs.iter() {
    //                result += &(i.to_string() + "\\n");
    //            }
    //        }
    //    }
    //
    //    result
    //}
    //
    //
}

/// INFO: This impl block is denoted to be about SSA
impl CFG {
    /// A map that maps a variable to all the block that defines it
    pub fn global_variables(&mut self) -> (BTreeSet<String>, BTreeMap<String, BTreeSet<BlockID>>) {
        // For each blocks,
        //   For each def in each block
        //     Let a particular def be about v
        //     Add def[v].insert(block)
        let mut globals = BTreeSet::<String>::new();
        let mut blocks = BTreeMap::<String, BTreeSet<BlockID>>::new();
        for (_, bbrc) in self.hm.iter() {
            let mut var_kill = BTreeSet::<String>::new();

            for ilb in bbrc.borrow().instrs.iter() {
                if let InstructionOrLabel::Instruction(i) = ilb {
                    if i.dest.is_none() || i.args.is_none() {
                        continue;
                    }
                    if let Some(args) = &i.args {
                        for arg in args {
                            if !var_kill.contains(arg) {
                                globals.insert(arg.clone());
                            }
                        }
                    }
                    if let Some(dest) = &i.dest {
                        var_kill.insert(dest.clone());

                        blocks
                            .entry(dest.clone())
                            .or_default()
                            .insert(bbrc.borrow().id);
                    }
                } else {
                    continue;
                };
            }
        }

        eprintln!("Set of global variables: {:?}", globals);
        eprintln!("Map of global variables to blocks: {:?}", blocks);
        (globals, blocks)
    }
    pub fn place_phi_functions_and_generate_ssa(&mut self) {
        let (globals, defs) = self.global_variables();

        let dff = DominanceDataFlow::new(self);
        let df = dff.df.clone();

        // INFO: A function to place phi functions down
        let mut place_phi_functions = || {
            for name in globals.clone() {
                let mut work_list = VecDeque::<usize>::new();

                if !defs.contains_key(&name) {
                    continue;
                }
                work_list.extend(defs.get(&name).unwrap().clone());

                while !work_list.is_empty() {
                    let block_id = work_list.pop_front().unwrap();

                    for d in df[&block_id].iter() {
                        let label = match self.id_to_bb[d].borrow().instrs.front().unwrap().clone()
                        {
                            InstructionOrLabel::Label(l) => InstructionOrLabel::Label(l),
                            _ => continue,
                        };
                        if self.id_to_bb[d].borrow().contains_phi_def(&name) {
                        } else {
                            let mut block_mut_b = self.id_to_bb[d].borrow_mut();
                            eprintln!("Constructing phi with  {name} from {label} at {}", d);
                            block_mut_b.insert_phi_def(&name, &mut self.instruction_counter);
                            work_list.push_back(*d);
                        }
                    }
                }
            }
        };
        place_phi_functions();

        let mut stack_of = BTreeMap::<String, Vec<String>>::new();
        for (def, _) in defs.iter() {
            stack_of.entry(def.clone()).or_insert(vec![def.clone()]);
        }
        let mut name_counter = BTreeMap::<String, usize>::new();

        // INFO: A function to rename operands in phi functions

        let mut map_from_id_to_instrs = BTreeMap::<usize, LinkedList<InstructionOrLabel>>::new();
        for (_, bb) in self.hm.iter() {
            map_from_id_to_instrs
                .entry(bb.borrow().id)
                .or_insert(bb.borrow().instrs.clone());
        }

        let mut new_to_old_names = BTreeMap::<String, String>::new();
        let mut rename_phi_defs = |stack_of: BTreeMap<String, Vec<String>>| {
            for (_, bb) in self.hm.iter() {
                if bb.borrow().func.is_none() {
                    continue;
                }
                bb.borrow_mut().rename_phi_def(
                    stack_of.clone(),
                    &dff.domtree,
                    &mut name_counter,
                    &self.id_to_bb,
                    &mut map_from_id_to_instrs,
                    &globals,
                    &mut new_to_old_names,
                )
            }
        };
        rename_phi_defs(stack_of);

        for (_, bb) in self.hm.iter_mut() {
            let id = bb.borrow().id;
            bb.borrow_mut().instrs = map_from_id_to_instrs.entry(id).or_default().clone();
        }
    }

    pub fn analyze_loop(&mut self) {
        let mut loops = Loops::new(self);

        for l in loops.loops.iter_mut() {
            self.dataflow(l)
        }
        // TODO:
        //
        // for l in loops
        //   cfg.dataflow(&mut l);
    }
}
