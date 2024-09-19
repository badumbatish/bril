use std::io::{self, Read};

use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Hash, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BrilType {
    // INFO: Technically we have a third option which is parameterized type
    Int,
    Bool,
}
// Define a structure to represent the JSON format
#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub op: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<Value>>,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bril_type: Option<BrilType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funcs: Option<Vec<String>>,
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
#[derive(Serialize, Deserialize, Debug)]
pub struct Function {
    pub name: String,

    pub instrs: Vec<InstructionOrLabel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<FunctionArg>>,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bril_type: Option<BrilType>,
    #[serde(flatten)]
    pub other_fields: Value, // Store unknown fields here
}

#[derive(Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<Function>,
    #[serde(flatten)]
    pub other_fields: Value, // Store unknown fields here
}

impl Instruction {
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
        match self.op.as_str() {
            "print" | "call" | "alloc" | "free" | "store" => true,
            _ => false,
        }
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
