import json
import sys
from utils import cfg

# Let's not optimize any function that jumps first.

# Meaning focus on purely nicely blocked function
def remove_dead_code(function):
    popped = 0
    used = set()

    for instruction in function["instrs"]:
        if "args" in instruction:
            for arg in instruction["args"]:
                used.add(arg)
    i = 0
    while i < len(function["instrs"]):
        instruction = function["instrs"][i]
        if "dest" in instruction and instruction["dest"] not in used:
            if "op" not in instruction or instruction["op"] != "call":
                function["instrs"].pop(i)
                popped += 1
                i -= 1
        
        i += 1
    
    return popped

def remove_dead_code_block(function, block):
    popped = 0
    unused_fornow = {}

    for inst in block:
        # check uses *before* processing defs
        for arg in inst.get("args", []):
            unused_fornow.pop(arg, None)

        if "dest" in inst:
            if inst["dest"] in unused_fornow:
                # if instruction defines something that hasn't been
                # used since its last def, remove the last def
                function["instrs"].remove(unused_fornow[inst["dest"]])
                popped += 1
            # mark this def as unused (at least for now)
            unused_fornow[inst["dest"]] = inst

    return popped

def trivial_dce_pass(prog):
    popped = 0
    for fn in prog["functions"]:
        popped += remove_dead_code(fn)
    return popped

def local_dce_pass(prog):
    popped = 0
    for fn in prog["functions"]:
        for block in cfg.make_basic_blocks(fn):
            popped += remove_dead_code_block(fn, block)
    return popped
            

if __name__ == "__main__":
    prog = json.load(sys.stdin)

    # trivial_dce_pass = lambda x: 0 # skip trivial dce
    # local_dce_pass = lambda x: 0 # skip local dce

    while local_dce_pass(prog) + trivial_dce_pass(prog) > 0:
        pass
    
    json.dump(prog, sys.stdout, indent=2)