use std::{
    fmt::Display,
    io::{self, Read},
};

use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize, Hash, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BrilType {
    // INFO: Technically we have a third option which is parameterized type
    Int,
    Bool,
    Float,
}

impl Display for BrilType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrilType::Int => write!(f, "int"),
            BrilType::Bool => write!(f, "bool"),
            BrilType::Float => write!(f, "float"),
        }
    }
}

// Define a structure to represent the JSON format
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub op: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bril_type: Option<BrilType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funcs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(flatten)]
    pub other_fields: Value, // Store unknown fields here
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label {
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionArg {
    pub name: String,
    #[serde(rename = "type")]
    pub fn_type: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
pub enum InstructionOrLabel {
    Label(Label),
    Instruction(Instruction),
}
impl InstructionOrLabel {
    pub fn new_phi(def: String) -> Self {
        InstructionOrLabel::Instruction(Instruction::new_phi(def))
    }
    pub fn new_dummy_head(header_name: String) -> Self {
        InstructionOrLabel::Label(Label { label: header_name })
    }
}
impl Display for InstructionOrLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionOrLabel::Label(lb) => write!(f, "{}", lb.clone().label),
            InstructionOrLabel::Instruction(ins) => write!(f, "{}", ins.clone().to_string()),
        }
    }
}

impl From<Label> for InstructionOrLabel {
    fn from(lb: Label) -> Self {
        Self::Label(lb)
    }
}
impl From<String> for InstructionOrLabel {
    fn from(lb: String) -> Self {
        Self::Label(Label { label: lb })
    }
}
impl From<&Option<InstructionOrLabel>> for InstructionOrLabel {
    fn from(lb: &Option<InstructionOrLabel>) -> Self {
        lb.clone().unwrap()
    }
}
impl From<Instruction> for InstructionOrLabel {
    fn from(lb: Instruction) -> Self {
        Self::Instruction(lb)
    }
}
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,

    pub instrs: Vec<InstructionOrLabel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<FunctionArg>>,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bril_type: Option<BrilType>,
    // #[serde(flatten)]
    // pub other_fields: Value, // Store unknown fields here
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<Function>,
    // #[serde(flatten)]
    // pub other_fields: Value, // Store unknown fields here
}

impl Instruction {
    pub fn new_phi(def: String) -> Self {
        Self {
            op: "phi".to_string(),
            dest: Some(def),
            args: Default::default(),
            bril_type: Default::default(),
            value: Default::default(),
            funcs: Default::default(),
            labels: Default::default(),
            other_fields: Default::default(),
        }
    }
    pub fn is_phi(&self) -> bool {
        &self.op == "phi"
    }
    pub fn is_add(&self) -> bool {
        &self.op == "add"
    }
    pub fn is_mul(&self) -> bool {
        &self.op == "mul"
    }
    pub fn is_sub(&self) -> bool {
        &self.op == "sub"
    }
    pub fn is_div(&self) -> bool {
        &self.op == "div"
    }
    pub fn is_eq(&self) -> bool {
        &self.op == "eq"
    }
    pub fn is_lt(&self) -> bool {
        &self.op == "lt"
    }
    pub fn is_gt(&self) -> bool {
        &self.op == "gt"
    }
    pub fn is_ge(&self) -> bool {
        &self.op == "ge"
    }
    pub fn is_le(&self) -> bool {
        &self.op == "le"
    }
    pub fn is_const(&self) -> bool {
        &self.op == "const"
    }
    pub fn is_id(&self) -> bool {
        &self.op == "id"
    }
    pub fn is_jmp(&self) -> bool {
        &self.op == "jmp"
    }
    pub fn is_br(&self) -> bool {
        &self.op == "br"
    }
    pub fn is_call(&self) -> bool {
        &self.op == "call"
    }
    pub fn is_ret(&self) -> bool {
        &self.op == "ret"
    }
    pub fn is_nop(&self) -> bool {
        &self.op == "nop"
    }
    pub fn is_print(&self) -> bool {
        &self.op == "print"
    }
    pub fn is_nonlinear(&self) -> bool {
        self.is_jmp() || self.is_call() || self.is_print() || self.is_br()
    }

    pub fn has_side_effects(&self) -> bool {
        matches!(
            self.op.as_str(),
            "print" | "call" | "alloc" | "free" | "store" | "ret"
        )
    }

    pub fn to_const_int(&mut self, i: u64) {
        self.op = "const".to_string();
        self.value = Some(json!(i));
    }

    /// this is for graphviz dot
    pub fn to_string(&self) -> String {
        // sum: int = add n five;
        if self.is_add() {
            format!(
                "{}: {} = {} {} {};",
                self.dest.clone().unwrap(),
                self.bril_type.clone().unwrap(),
                self.op.replace("\"", ""),
                self.args.clone().unwrap()[0].to_string().replace("\"", ""),
                self.args.clone().unwrap()[1].to_string().replace("\"", "")
            )
        } else if self.is_const() {
            return format!(
                "{}: {} = {} {};",
                self.dest.clone().unwrap(),
                self.bril_type.clone().unwrap(),
                self.op,
                self.value.clone().unwrap(),
            );
        } else if self.is_ret() {
            return format!(
                "{} {};",
                self.op,
                self.args.clone().unwrap()[0].to_string().replace("\"", ""),
            );
        } else if self.is_call() {
            // let dest = match &self.dest {
            //     Some(k) => format!("{} :", k.clone()),
            //     None => "".to_string(),
            // };
            //
            // let acl = self.args.clone();
            // let one = match &acl {
            //     Some(args) => args[0].to_string(),
            //     None => "".to_string(),
            // };
            // let two = match &acl {
            //     Some(args) => args[1].to_string(),
            //     None => "".to_string(),
            // };
            return "...call func".to_string();
        } else if self.is_print() {
            return "print ...".to_string();
        } else {
            "default".to_string()
        }

        // TODO: support const, call, ret, add
    }
}
impl Function {}
impl Program {
    pub fn stdin() -> Self {
        // Read input from stdin
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .expect("Failed to read input");

        // Deserialize JSON into the Program structure
        serde_json::from_str(&input).expect("Failed to parse JSON")
    }

    pub fn stdout(&self) {
        serde_json::to_writer_pretty(io::stdout(), &self).expect("Failed to write JSON");
    }
}
