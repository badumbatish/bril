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

# Local DCE pseudocode from lecture
# unused: {var: inst} = {}
# for inst in block:
#     # if it's used, it's not unused
#     for use in inst.uses:
#         del unused[use]
#     if inst.dest 
#         # if this inst defines something
#         # we can kill the unused definition
#         if unused[inst.dest]:
#             remove unused[inst.dest]
#         # mark this def as unused for now
#         unused[inst.dest] = inst

def remove_dead_code_block(block):
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
                block["instrs"].pop(unused_fornow[inst["dest"]])
                popped += 1
            # mark this def as unused (at least for now)
            unused_fornow[inst["dest"]] = inst

    return popped
            

if __name__ == "__main__":
    prog = json.load(sys.stdin)

    # basic blockwise local DCE
    for function in prog["functions"]:
        for block in cfg.make_basic_blocks(function):
            while remove_dead_code_block(block):
                pass
    
    # trivial DCE
    for function in prog["functions"]:
        while remove_dead_code(function):
            pass

    json.dump(prog, sys.stdout, indent=2)