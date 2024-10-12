use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use crate::basic_block::BasicBlock;

pub type BlockID = usize;
pub type IdToBbMap = HashMap<BlockID, Rc<RefCell<BasicBlock>>>;
pub type DomTree = BTreeMap<BlockID, BlockID>;
pub type SSANameStack = BTreeMap<String, Vec<String>>;
pub type BbPtr = Rc<RefCell<BasicBlock>>;
pub type NameCounter = BTreeMap<String, usize>;
