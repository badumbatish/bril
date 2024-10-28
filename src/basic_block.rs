use crate::{
    aliases::{BlockID, DomTree, IdToBbMap, SSANameStack},
    bril_syntax::{Function, InstructionOrLabel, Label},
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, LinkedList, VecDeque},
    hash::{Hash, Hasher},
    rc::Rc,
};

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
    pub instrs: LinkedList<InstructionOrLabel>,
    pub predecessors: Vec<Rc<RefCell<BasicBlock>>>,
    pub successors: Vec<Rc<RefCell<BasicBlock>>>,
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
    pub fn push_front(&mut self, ilb: &InstructionOrLabel) {
        todo!();
    }
    pub fn push_before_header(&mut self, lib: &InstructionOrLabel) {}
    pub fn push_back(&mut self, ilb: &InstructionOrLabel) {
        self.instrs.push_back(ilb.clone());
    }

    pub fn insert_at(&mut self, position: usize, ilb: &InstructionOrLabel) {
        let mut tail = self.instrs.split_off(position);

        self.instrs.push_back(ilb.clone()); // Manually walk the iterator to the desired position
        self.instrs.append(&mut tail);
    }
    pub fn get_label(&self) -> String {
        if self.instrs.is_empty() {
            unreachable!()
        }

        let a = self.instrs.front().unwrap();

        match a {
            InstructionOrLabel::Label(l) => l.label.to_string(),
            _ => "".to_string(),
        }
    }

    fn new_name(
        var: &String,
        stack_of: &mut SSANameStack,
        name_counter: &mut BTreeMap<String, usize>,
        new_to_old_names: &mut BTreeMap<String, String>,
    ) -> String {
        let fresh = var.clone() + &name_counter.get(var).unwrap_or(&0).to_string();
        *name_counter.entry(var.clone()).or_insert(0) += 1;

        stack_of.entry(var.clone()).or_default().push(fresh.clone());

        new_to_old_names.insert(fresh.clone(), var.clone());
        fresh
    }
    pub fn rename_phi_def(
        &self,
        mut stack_of: SSANameStack,
        dom_tree: &DomTree,
        name_counter: &mut BTreeMap<String, usize>,
        id_to_bb: &IdToBbMap,
        id_to_ins: &mut BTreeMap<usize, LinkedList<InstructionOrLabel>>,
        globals: &BTreeSet<String>,
        new_to_old_names: &mut BTreeMap<String, String>,
    ) {
        // INFO: Rename phi function first
        for inst in id_to_ins.entry(self.id).or_default().iter_mut() {
            if let InstructionOrLabel::Instruction(i) = inst {
                if i.is_phi() {
                    if let Some(dest) = &mut i.dest {
                        *dest = BasicBlock::new_name(
                            dest,
                            &mut stack_of,
                            name_counter,
                            new_to_old_names,
                        );
                    }
                }
            }
        }

        for inst in id_to_ins.entry(self.id).or_default().iter_mut() {
            // Rename arguments of the instruction
            if let InstructionOrLabel::Instruction(i) = inst {
                if i.is_phi() {
                    continue;
                }
                if let Some(args) = &mut i.args {
                    for arg in args.iter_mut() {
                        //eprintln!("Before, arg = {}", arg);
                        *arg = stack_of
                            .entry(arg.clone())
                            .or_insert(vec![arg.clone()])
                            .last()
                            .unwrap()
                            .clone();
                        if globals.contains(arg) {}
                        //eprintln!("After, arg = {}", arg);
                    }
                }
                if let Some(dest) = &mut i.dest {
                    *dest =
                        BasicBlock::new_name(dest, &mut stack_of, name_counter, new_to_old_names);
                    if globals.contains(dest) {
                        // Update i.dest with the fresh name
                    }
                }
            }
        }
        for succ in self.successors.iter() {
            for instr in id_to_ins.entry(succ.borrow().id).or_default().iter_mut() {
                if let InstructionOrLabel::Instruction(i) = instr {
                    if i.is_phi() {
                        let v = &i.dest.clone().unwrap();
                        eprintln!("v: {v}");
                        let v = if let Some(old_name) = new_to_old_names.get(v) {
                            eprintln!(
                                "Transformed name from name maps: {}",
                                stack_of[old_name].last().unwrap()
                            );
                            stack_of[old_name].last().unwrap().clone()
                        } else {
                            eprintln!(
                                "Transformed name from stack: {}",
                                stack_of[v].last().unwrap()
                            );
                            stack_of[v].last().unwrap().clone()
                        };
                        let label = self.get_label();
                        if i.labels.is_none() {
                            i.labels = Some(Vec::new());
                        }
                        if i.args.is_none() {
                            i.args = Some(Vec::new());
                        }

                        if let Some(args) = &mut i.args {
                            args.push(v.clone());
                            //args.push(stack_of[v].last().unwrap().clone());
                        }
                        if let Some(labels) = &mut i.labels {
                            labels.push(label.clone());
                        }

                        eprintln!("Inserting {v} with {label} into {}", succ.borrow().id);
                    }
                }
            }
        }
        for (a, b) in dom_tree.iter() {
            if *b == self.id && b != a {
                id_to_bb[a].borrow().rename_phi_def(
                    stack_of.clone(),
                    dom_tree,
                    name_counter,
                    id_to_bb,
                    id_to_ins,
                    globals,
                    new_to_old_names,
                )
            }
        }
    }
    pub fn insert_phi_def(&mut self, def: &String, instruction_counter: &mut usize) {
        //eprintln!("Insert phi def into {}", self.id);
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
                        //p.labels.as_mut().unwrap().push(label.to_string());
                        //p.args.as_mut().unwrap().push(def.to_string());
                        return;
                    }
                }
                _ => continue,
            }
        }

        // INFO: At this point we don't see any phi related to our def
        // We create our phi and def
        //
        //self.push_back(&InstructionOrLabel::new_phi(
        //    def.clone(),
        //    instruction_counter,
        //)); // Insert the new element at the current iterator position
        let mut p = InstructionOrLabel::new_phi(def.clone(), instruction_counter);
        if let InstructionOrLabel::Instruction(ref mut p) = p {
            if p.labels.is_none() {
                p.labels = Some(Vec::new());
            }
            if p.args.is_none() {
                p.args = Some(Vec::new());
            }
            //p.labels.as_mut().unwrap().push(label.to_string());
            //p.args.as_mut().unwrap().push(def.to_string());
        }
        self.insert_at(1, &p); // Insert the new element at the current iterator position
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
    pub fn contains_phi_def(&self, def: &String) -> bool {
        self.instrs.iter().any(|x| {
            if let InstructionOrLabel::Instruction(i) = x {
                i.is_phi()
                    && i.dest.is_some()
                    && i.dest.clone().unwrap() == *def
                    && i.labels.is_some()
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
    pub fn default_with_label(id: BlockID, label: &String) -> BasicBlock {
        let mut result = Self::default(id);

        result.push_back(&InstructionOrLabel::Label(Label {
            label: label.clone(),
        }));
        result
    }

    pub fn simple_basic_blocks_vec_from_function(
        f: &Function,
        block_id: &mut BlockID,
    ) -> Vec<Rc<RefCell<BasicBlock>>> {
        let mut result: Vec<Rc<RefCell<BasicBlock>>> = Vec::new();
        let mut i = 0;
        let entry_bb = BasicBlock::default(*block_id);
        let entry_bb_rcf: Rc<RefCell<BasicBlock>> = Rc::<RefCell<BasicBlock>>::new(entry_bb.into());
        let entry_header_name = "entry".to_string() + &f.name;

        entry_bb_rcf
            .borrow_mut()
            .instrs
            .push_back(InstructionOrLabel::new_dummy_head(entry_header_name));
        *block_id += 1;
        entry_bb_rcf.borrow_mut().func = Some(f.clone());
        result.push(entry_bb_rcf);
        // let mut last_instruction_before_construction = 0;
        let mut non_linear_before = false;
        while i < f.instrs.len() {
            // this match only happens if instruction is at start of function or after a branch
            // without label
            let b: BasicBlock = BasicBlock::default(*block_id);
            let bb: Rc<RefCell<BasicBlock>> = Rc::<RefCell<BasicBlock>>::new(b.into());
            *block_id += 1;
            if !non_linear_before {
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
                        if instr.instruction_id.is_none() {
                            eprintln!("This has None : {:?}", instr);
                            panic!();
                        }
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
