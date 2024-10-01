import json
import sys
from utils import cfg

class BasicBlock:
    def __init__(self, label: str = "", instrs: list = []):
        self.preds: list[BasicBlock] = []
        self.label: str = label
        self.instrs: list = instrs

class Phi:
    def __init__(self, block: BasicBlock):
        self.block: BasicBlock = block
        self.labels: list[str] = []
        self.arguments: list[str] = []
        pass

    def append_operand(self, label: str, rhs: str):
        self.labels.append(label)
        self.arguments.append(rhs)

cnt: int = 0
current_def: dict[str, str] = {}
incomplete_phis: dict[BasicBlock, dict[str, Phi]] = {}
sealed_blocks: set[BasicBlock] = set()
label_to_block = dict[str, BasicBlock] = {}

def write_variable(var: str, block: BasicBlock, val):
    current_def[var][block] = val

def read_variable(var: str, block: BasicBlock):
    if block in current_def[var]:
        # lvn case
        return current_def[var][block]
    # not locally defined, global value numbering
    return read_variable_recursive(var, block)

def read_variable_recursive(var: str, block: BasicBlock):
    if block not in sealed_blocks:
        val = Phi(block)
        incomplete_phis[block][var] = val
    elif len(block.preds) == 1:
        val = read_variable(var, block)
    else:
        val = Phi(block)
        write_variable(var, block, val)
        val = add_phi_operands(var, val)

def add_phi_operands(var: str, phi: Phi):
    for pred in phi.block.preds:
        phi.append_operand(pred.label, read_variable(var, pred))
    return try_remove_trivial_phi(phi)

def try_remove_trivial_phi(phi: Phi) -> Phi:
    # for now
    return phi

def seal_block(block: BasicBlock):
    for var, phi in incomplete_phis[block]:
        add_phi_operands(var, phi)
    sealed_blocks.add(block)


def should_keep(instr):
    # not all instructions have an 'op' field, like labels
    if "op" not in instr:
        return True
    return instr["op"] != "nop"

def basic_blocks_ssa(func):
    blocks: list[BasicBlock] = []
    curr_block: BasicBlock = None



    for inst in func["instrs"]:
        if "label" in inst:
            if curr_block:
                blocks.append(curr_block)
                curr_block = label_to_block.get(inst["label"], BasicBlock())
            curr_block.label = inst["label"]
            curr_block.instrs.append(inst)
        else:  # if not label, must be op
            if curr_block is None:  # no entry label
                curr_block = BasicBlock("", []) # empty label, add placeholders at end if needed
            curr_block.instrs.append(inst)
            if inst["op"] in cfg.branch_instructions:
                
                yield curr_block
                curr_block = None

    if curr_block:
        blocks.append(curr_block)
    return blocks

if __name__ == "__main__":
    prog = json.load(sys.stdin)
    for fn in prog["functions"]:
        fn["instrs"] = [instr for instr in fn["instrs"] if should_keep(instr)]
    json.dump(prog, sys.stdout, indent=2)
