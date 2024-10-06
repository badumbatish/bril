use crate::dominance::DominanceDataFlow;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    fmt::Debug,
    hash::{Hash, Hasher},
    rc::Rc,
};
pub type BlockID = usize;
use crate::bril_syntax::{Function, InstructionOrLabel, Program};

// Maybe this will be useful in the future but for now a leader is the first instruction in the
// block
//#[derive(Hash, Debug, Eq, PartialEq, Clone)]
//pub enum Leader {
//    FunctionName(Function),
//    InstructionOrLabel(InstructionOrLabel),
//}
//
//impl Leader {
//    pub fn from_label_string(label: String) -> Self {
//        Self::InstructionOrLabel(InstructionOrLabel::Label(Label { label }))
//    }
//
//    pub fn from_label(label: Label) -> Self {
//        Self::InstructionOrLabel(InstructionOrLabel::Label(label))
//    }
//
//    pub fn from_instr_or_label(instr: InstructionOrLabel) -> Self {
//        Self::InstructionOrLabel(instr)
//    }
//}
pub struct BasicBlock {
    pub func: Option<Function>,
    pub id: BlockID,
    pub instrs: VecDeque<InstructionOrLabel>,
    pub predecessors: Vec<Rc<RefCell<BasicBlock>>>,
    pub successors: Vec<Rc<RefCell<BasicBlock>>>,
}

#[derive(Debug, PartialEq)]
pub enum DataFlowDirection {
    Forward,
    Backward,
}
#[derive(Debug, PartialEq)]
pub enum TransferResult {
    Changed,
    NonChanged,
}
#[derive(Debug, PartialEq)]
pub enum ConditionalTransferResult {
    AllPathTaken,
    FirstPathTaken,
    SecondPathTaken,
    NoPathTaken,
}
#[derive(Debug, PartialEq)]
pub enum DataFlowOrder {
    EntryNodesOnly,
    PostOrderDFS,
    BFS,
}
pub trait DataFlowAnalysis {
    fn meet(&mut self, bb: &mut BasicBlock);
    fn transfer(&mut self, bb: &mut BasicBlock) -> TransferResult;
    fn transform(&mut self, bb: &mut BasicBlock);
    fn get_dataflow_direction(&self) -> DataFlowDirection;
    fn get_dataflow_order(&self) -> DataFlowOrder;
}

pub trait ConditionalDataFlowAnalysis {
    fn meet(&mut self, bb: &mut BasicBlock);
    fn transfer(&mut self, bb: &mut BasicBlock) -> ConditionalTransferResult;
    fn transform(&mut self, bb: &mut BasicBlock);
}
impl std::fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "----Basic Block----
----Instructions: \n",
        )
        .unwrap();
        for instr in self.instrs.iter() {
            writeln!(f, "{:?}", instr).unwrap();
        }
        writeln!(f, "Pred: ").unwrap();
        for instr in self.predecessors.iter() {
            writeln!(f, "{:?}", instr.borrow()).unwrap();
        }
        writeln!(f, "Succ: ").unwrap();
        for instr in self.successors.iter() {
            writeln!(f, "{:?}", instr.borrow()).unwrap();
        }
        writeln!(f, "\n")
    }
}

impl Hash for BasicBlock {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for BasicBlock {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BasicBlock {}

impl BasicBlock {
    pub fn get_label(&self) -> String {
        if self.instrs.is_empty() {
            unreachable!()
        }

        let a = self.instrs.front().unwrap();

        match a {
            InstructionOrLabel::Label(l) => l.label.to_string(),
            _ => {
                "".to_string()
            }
        }
    }
    pub fn rename_phi_def(
        &mut self,
        mut stack_of: BTreeMap<String, Vec<String>>,
        dom_tree: &BTreeMap<BlockID, BlockID>,
        name_counter: &mut BTreeMap<String, usize>,
        id_to_bb: &HashMap<BlockID, Rc<RefCell<BasicBlock>>>,
    ) {
        //  stack[var] = [] # stack of names for each variable
        //dom_tree[b] = list of children of block b in the dominator tree
        //              i.e., blocks that are *immediately* dominated by b
        //def rename(block):
        //    remember the stack
        //
        //    for inst in block:
        //        inst.args = [stack[arg].top for arg in inst.args]
        //        fresh = fresh_name(inst.dest)
        //        stack[inst.dest].push(fresh)
        //        inst.dest = fresh
        //    for succ in block.successors:
        //        for phi in succ.phis:
        //            v = phi.dest
        //            update the arg in this phi corresponding to block to stack[v].top
        //    for child in dom_tree[block]:
        //        rename(child)
        //
        //    restore the stack by popping what we pushed
        //
        eprintln!("Rename with SSA from {}", self.id);
        let dest_later = String::new();
        for inst in self.instrs.iter_mut() {
            // Rename arguments of the instruction
            if let InstructionOrLabel::Instruction(i) = inst {
                if let Some(args) = &mut i.args {
                    for arg in args.iter_mut() {
                        *arg = stack_of
                            .entry(arg.clone())
                            .or_insert(vec![arg.clone()])
                            .last()
                            .unwrap()
                            .clone();
                    }
                    if let Some(dest) = &mut i.dest {
                        let fresh =
                            dest.clone() + &name_counter.get(dest).unwrap_or(&0).to_string();
                        *name_counter.entry(dest.clone()).or_insert(0) += 1;

                        stack_of
                            .entry(dest.clone())
                            .or_default()
                            .push(fresh.clone());

                        eprintln!("Renaming to {fresh}");
                        *dest = fresh; // Update i.dest with the fresh name
                    }
                }
            }
        }
        for succ in self.successors.iter() {
            for instr in succ.borrow_mut().instrs.iter_mut() {
                if let InstructionOrLabel::Instruction(i) = instr {
                    if i.is_phi() {
                        let v = &i.dest.clone().unwrap();
                        if let Some(stack) = stack_of.get(v) {
                            if let Some(top) = stack.last() {
                                eprintln!(
                                    "I am at {} with {:?}",
                                    self.id,
                                    self.instrs.front().unwrap()
                                );

                                let label = self.get_label();
                                i.rename_phi(v.to_string(), top.to_string(), label);
                                // Update the phi argument for this block to the top of the stack
                            }
                        }
                    }
                }
            }
        }

        for (a, b) in dom_tree.iter() {
            if *b == self.id && b != a {
                id_to_bb[a].borrow_mut().rename_phi_def(
                    stack_of.clone(),
                    dom_tree,
                    name_counter,
                    id_to_bb,
                )
            }
        }
    }
    pub fn insert_phi_def(&mut self, def: &String, label: InstructionOrLabel) {
        eprintln!("Insert phi def into {}", self.id);
        for i in self.instrs.iter_mut() {
            match i {
                InstructionOrLabel::Instruction(p) => {
                    if p.is_phi() && p.dest.as_ref().unwrap() == def {
                        if p.labels.is_none() {
                            p.labels = Some(Vec::new());
                        }
                        if p.args.is_none() {
                            p.args = Some(Vec::new());
                        }
                        p.labels.as_mut().unwrap().push(label.to_string());
                        p.args.as_mut().unwrap().push(def.to_string());
                        return;
                    }
                }
                _ => continue,
            }
        }

        self.instrs
            .insert(1, InstructionOrLabel::new_phi(def.clone()));

        for i in self.instrs.iter_mut() {
            match i {
                InstructionOrLabel::Instruction(p) => {
                    if p.is_phi() && p.dest.as_ref().unwrap() == def {
                        if p.labels.is_none() {
                            p.labels = Some(Vec::new());
                        }
                        if p.args.is_none() {
                            p.args = Some(Vec::new());
                        }
                        p.labels.as_mut().unwrap().push(label.to_string());
                        p.args.as_mut().unwrap().push(def.to_string());
                        return;
                    }
                }
                _ => continue,
            }
        }
    }

    // Contains empty phi def
    pub fn contains_empty_phi_def(&self, def: &String) -> bool {
        self.instrs.iter().any(|x| {
            if let InstructionOrLabel::Instruction(i) = x {
                i.is_phi()
                    && i.dest.is_some()
                    && i.dest.clone().unwrap() == *def
                    && (i.labels.is_none() || i.labels.clone().unwrap().is_empty())
            } else {
                false
            }
        })
    }
    // Check if the current block contains any phi definition about def variable
    pub fn contains_phi_def(&self, def: &String, label: InstructionOrLabel) -> bool {
        self.instrs.iter().any(|x| {
            if let InstructionOrLabel::Instruction(i) = x {
                i.is_phi()
                    && i.dest.is_some()
                    && i.dest.clone().unwrap() == *def
                    && i.labels.is_some()
                    && i.labels.clone().unwrap().contains(&label.to_string())
            } else {
                false
            }
        })
    }
    pub fn get_definitions(&self) -> Vec<InstructionOrLabel> {
        self.instrs
            .clone()
            .into_iter()
            .filter(|x| {
                if let InstructionOrLabel::Instruction(i) = x {
                    i.dest.is_some()
                } else {
                    false
                }
            })
            .collect()
    }
    pub fn as_txt_instructions(self) -> String {
        let _result = String::new();
        todo!()
    }
    pub fn default(id: BlockID) -> BasicBlock {
        Self {
            func: None,
            id,
            instrs: Default::default(),
            predecessors: Default::default(),
            successors: Default::default(),
        }
    }

    pub fn simple_basic_blocks_vec_from_function(
        f: Function,
        id: &mut BlockID,
    ) -> Vec<Rc<RefCell<BasicBlock>>> {
        let mut result: Vec<Rc<RefCell<BasicBlock>>> = Vec::new();
        let mut i = 0;
        let entry_bb = BasicBlock::default(*id);
        let entry_bb_rcf: Rc<RefCell<BasicBlock>> = Rc::<RefCell<BasicBlock>>::new(entry_bb.into());
        let entry_header_name = "entry".to_string() + &f.name;

        entry_bb_rcf
            .borrow_mut()
            .instrs
            .push_back(InstructionOrLabel::new_dummy_head(entry_header_name));
        *id += 1;
        entry_bb_rcf.borrow_mut().func = Some(f.clone());
        result.push(entry_bb_rcf);
        // let mut last_instruction_before_construction = 0;
        let mut non_linear_before = false;
        while i < f.instrs.len() {
            // this match only happens if instruction is at start of function or after a branch
            // without label
            let b: BasicBlock = BasicBlock::default(*id);
            let bb: Rc<RefCell<BasicBlock>> = Rc::<RefCell<BasicBlock>>::new(b.into());
            *id += 1;
            if result.is_empty() {
                bb.borrow_mut().func = Some(f.clone());
            } else if !non_linear_before {
                bb.borrow_mut()
                    .predecessors
                    .push(result.last().unwrap().clone());
                result
                    .last_mut()
                    .unwrap()
                    .borrow_mut()
                    .successors
                    .push(bb.clone());
                non_linear_before = true;
            }

            let mut bb_mut = bb.borrow_mut();
            bb_mut.instrs.push_back(f.instrs[i].clone());
            i += 1;
            loop {
                if i >= f.instrs.len() {
                    break;
                }
                match &f.instrs[i] {
                    InstructionOrLabel::Instruction(instr) => {
                        bb_mut
                            .instrs
                            .push_back(InstructionOrLabel::Instruction(instr.clone()));
                        if instr.is_jmp() || instr.is_br() {
                            non_linear_before = true;
                            break;
                        }
                    }
                    // TODO: Handle doubly label
                    InstructionOrLabel::Label(_) => {
                        i -= 1;
                        non_linear_before = false;
                        break;
                    }
                }
                i += 1;
            }

            result.push(bb.clone());
            i += 1;
        }
        result
    }
}
#[derive(Debug)]
pub struct CFG {
    pub hm: HashMap<InstructionOrLabel, Rc<RefCell<BasicBlock>>>,
    pub id_to_bb: HashMap<BlockID, Rc<RefCell<BasicBlock>>>,
}
// main:
// @main
impl CFG {
    //pub fn components_from_function(
    //    f: Function,
    //) -> (
    //    HashMap<InstructionOrLabel, Rc<RefCell<BasicBlock>>>,
    //    HashMap<BlockID, Rc<RefCell<BasicBlock>>>,
    //) {
    //    // O(n)
    //    let mut hm = HashMap::<InstructionOrLabel, Rc<RefCell<BasicBlock>>>::new();
    //    let mut id_to_bb = HashMap::<BlockID, Rc<RefCell<BasicBlock>>>::new();
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
    pub fn from_program(p: Program) -> Self {
        let mut hm = HashMap::<InstructionOrLabel, Rc<RefCell<BasicBlock>>>::new();

        let mut id_to_bb = HashMap::<BlockID, Rc<RefCell<BasicBlock>>>::new();
        let mut basic_block_counter = 0;
        // Iterate to put basic blocks into the graph
        for func in p.functions {
            let simple_basic_blocks_vec_from_function =
                BasicBlock::simple_basic_blocks_vec_from_function(func, &mut basic_block_counter);
            for bb in simple_basic_blocks_vec_from_function {
                hm.insert(bb.borrow().instrs.front().unwrap().clone(), bb.clone());
                id_to_bb.insert(bb.borrow().id, bb.clone());
            }
        }
        // Iterate to connect them
        for i in hm.values() {
            let mut bi = i.borrow_mut();
            if !bi.instrs.is_empty() {
                match bi.instrs.clone().into_iter().last() {
                    Some(instr) => match instr {
                        InstructionOrLabel::Label(_) => {
                            //eprintln!("This should not happen in CFG::from_program")
                        }
                        InstructionOrLabel::Instruction(ins) => {
                            if ins.is_br() {
                                bi.successors.push(
                                    hm[&InstructionOrLabel::from(
                                        ins.labels.clone().unwrap()[1].clone(),
                                    )]
                                        .clone(),
                                );
                                bi.successors.push(
                                    hm[&InstructionOrLabel::from(
                                        ins.labels.clone().unwrap()[0].clone(),
                                    )]
                                        .clone(),
                                );

                                hm[&InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[1].clone(),
                                )]
                                    .borrow_mut()
                                    .predecessors
                                    .push(i.clone());
                                hm[&InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[0].clone(),
                                )]
                                    .borrow_mut()
                                    .predecessors
                                    .push(i.clone());
                            } else if ins.is_jmp() {
                                bi.successors.push(
                                    hm[&InstructionOrLabel::from(
                                        ins.labels.clone().unwrap()[0].clone(),
                                    )]
                                        .clone(),
                                );
                                hm[&InstructionOrLabel::from(
                                    ins.labels.clone().unwrap()[0].clone(),
                                )]
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

        Self { hm, id_to_bb }
    }

    pub fn to_program(&self) -> Program {
        let mut p = Program {
            functions: Vec::default(),
            // other_fields: serde_json::Value::default(),
        };

        // The function is here just because we want to maintain the initial order of function in
        // textual IR
        let mut sorted_by_func_id: Vec<Rc<RefCell<BasicBlock>>> =
            self.hm.values().cloned().collect();

        // Sort the vector by the 'id' field
        sorted_by_func_id.sort_by_key(|e| e.borrow().id);

        for i in sorted_by_func_id {
            if i.borrow().func.is_some() {
                p.functions.push(self.insert_instr_func(i.clone()));
            }
        }
        p
    }

    fn insert_instr_func(&self, bb: Rc<RefCell<BasicBlock>>) -> Function {
        let mut visited = HashSet::<BlockID>::default();
        let mut q = VecDeque::<Rc<RefCell<BasicBlock>>>::default();
        let mut vec_instr = Vec::<InstructionOrLabel>::new();
        q.push_back(bb.clone());
        visited.insert(bb.borrow().id);
        let mut v = Vec::default();
        v.push(bb.clone());
        while !q.is_empty() {
            let visit_bb = q.pop_front().unwrap();

            for succ in visit_bb.borrow().successors.iter().rev() {
                let a = succ.borrow().id;
                if !visited.contains(&a) {
                    q.push_back(succ.clone());
                    v.push(succ.clone());
                    visited.insert(a);
                }
            }
        }
        v.sort_by_key(|bb| bb.borrow().id);

        for bb in v {
            vec_instr.extend(bb.borrow().instrs.clone());
        }
        let mut func = bb.borrow().func.clone().unwrap().clone();
        func.instrs = vec_instr;

        func
    }

    pub fn print_hm(&self) {
        for i in self.hm.iter() {
            eprintln!("{:?}", i.0);
            eprintln!("{:?}", i.1.borrow())
        }
    }
    pub fn dataflow_forward_optimistically(&self, d: &mut impl ConditionalDataFlowAnalysis) {
        // do the dataflow optimistically
        //
        //
        let mut q = VecDeque::<Rc<RefCell<BasicBlock>>>::default();
        for i in self.hm.clone() {
            if i.1.borrow_mut().func.is_some() {
                q.push_back(i.1.clone());
            }
            while !q.is_empty() {
                let visit_bb = q.pop_front().expect("hi").clone();
                // visit_bb.borrow_mut();
                d.meet(&mut visit_bb.borrow_mut());

                let transfer_result = d.transfer(&mut visit_bb.borrow_mut());
                match transfer_result {
                    ConditionalTransferResult::AllPathTaken => {
                        q.extend(visit_bb.borrow_mut().successors.clone())
                    }
                    ConditionalTransferResult::FirstPathTaken => {
                        //eprintln!("First path taken, from {}", s);
                        q.push_back(visit_bb.borrow_mut().successors[0].clone())
                    }
                    ConditionalTransferResult::SecondPathTaken => {
                        q.push_back(visit_bb.borrow_mut().successors[1].clone())
                    }
                    ConditionalTransferResult::NoPathTaken => continue,
                };
            }
        }

        for i in self.hm.iter() {
            d.transform(&mut i.1.borrow_mut())
        }
    }

    pub fn dataflow(&self, d: &mut impl DataFlowAnalysis) {
        // do the dataflow
        //
        //
        let mut q = VecDeque::<Rc<RefCell<BasicBlock>>>::default();
        for i in self.hm.clone() {
            if i.1.borrow_mut().func.is_some() {
                match d.get_dataflow_order() {
                    DataFlowOrder::EntryNodesOnly => q.push_back(i.1.clone()),
                    DataFlowOrder::BFS => q.extend(Self::bfs_children(&mut i.1.clone())),
                    DataFlowOrder::PostOrderDFS => todo!(),
                }
                while !q.is_empty() {
                    let visit_bb = q.pop_front().expect("hi").clone();
                    // visit_bb.borrow_mut();
                    d.meet(&mut visit_bb.borrow_mut());

                    if d.transfer(&mut visit_bb.borrow_mut()) == TransferResult::Changed {
                        if d.get_dataflow_direction() == DataFlowDirection::Forward {
                            q.extend(visit_bb.borrow_mut().successors.clone());
                        } else if d.get_dataflow_direction() == DataFlowDirection::Backward {
                            q.extend(visit_bb.borrow_mut().predecessors.clone());
                        }
                    }
                }
            }
        }

        for i in self.hm.iter() {
            d.transform(&mut i.1.borrow_mut())
        }
    }

    // fn dfs_children(bb: &mut BasicBlock<T>) {}
    fn bfs_children(bb: &mut Rc<RefCell<BasicBlock>>) -> VecDeque<Rc<RefCell<BasicBlock>>> {
        let mut visited = HashSet::<InstructionOrLabel>::default();
        let mut q = VecDeque::<Rc<RefCell<BasicBlock>>>::default();
        let mut result = VecDeque::<Rc<RefCell<BasicBlock>>>::default();

        q.push_back(bb.clone());
        result.push_back(bb.clone());

        visited.insert(bb.borrow().instrs.front().unwrap().clone());
        while !q.is_empty() {
            let visit_bb = q.pop_front().unwrap();
            result.push_back(visit_bb.clone());
            for succ in visit_bb.borrow().successors.iter().rev() {
                let a = succ.borrow().instrs.front().unwrap().clone();
                if !visited.contains(&a) {
                    q.push_back(succ.clone());
                    visited.insert(a);
                }
            }
        }

        result
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
    //fn make_info_from_block(bb: Rc<RefCell<BasicBlock>>) -> String {
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

impl CFG {
    /// A map that maps a variable to all the block that defines it
    pub fn def_variables(&mut self) -> BTreeMap<String, BTreeSet<BlockID>> {
        // For each blocks,
        //   For each def in each block
        //     Let a particular def be about v
        //     Add def[v].insert(block)
        let mut result = BTreeMap::<String, BTreeSet<BlockID>>::new();
        for (_, bbrc) in self.hm.iter() {
            for d in bbrc.borrow().get_definitions() {
                if let InstructionOrLabel::Instruction(i) = d {
                    result
                        .entry(i.dest.clone().unwrap())
                        .or_default()
                        .insert(bbrc.borrow().id);
                } else {
                    continue;
                };
            }
        }
        result
    }
    pub fn place_phi_functions_and_generate_ssa(&mut self) {
        let defs = self.def_variables();

        let mut dff = DominanceDataFlow::new(self);
        self.dataflow(&mut dff);
        let df = dff.infer(self).df.clone();

        self.place_phi_functions(&defs, &df);

        let mut stack_of = BTreeMap::<String, Vec<String>>::new();
        for (def, _) in defs.iter() {
            stack_of.entry(def.clone()).or_insert(vec![def.clone()]);
        }
        let mut name_counter = BTreeMap::<String, usize>::new();
        self.rename_phi_defs(stack_of, dff.domtree, &mut name_counter);
    }

    pub fn place_phi_functions(
        &mut self,
        defs: &BTreeMap<String, BTreeSet<BlockID>>,
        dominance_frontier: &BTreeMap<BlockID, HashSet<BlockID>>,
    ) {
        for (var, defs_of_var) in defs.iter() {
            for defining_block in defs_of_var.iter() {
                for block in dominance_frontier[defining_block].iter() {
                    let label = self.id_to_bb[defining_block]
                        .borrow()
                        .instrs
                        .front()
                        .unwrap()
                        .clone();
                    if self.id_to_bb[block]
                        .borrow()
                        .contains_phi_def(var, label.clone())
                    {
                        continue;
                    } else {
                        let mut block_mut_b = self.id_to_bb[block].borrow_mut();

                        block_mut_b.insert_phi_def(var, label);
                    }
                }
            }
        }
    }

    pub fn rename_phi_defs(
        &mut self,
        stack_of: BTreeMap<String, Vec<String>>,
        dom_tree: BTreeMap<BlockID, BlockID>,
        name_counter: &mut BTreeMap<String, usize>,
    ) {
        for (_, bb) in self.hm.iter() {
            if bb.borrow().func.is_none() {
                continue;
            }
            bb.borrow_mut().rename_phi_def(
                stack_of.clone(),
                &dom_tree,
                name_counter,
                &self.id_to_bb,
            )
        }
    }
}
