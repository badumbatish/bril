import json
import sys

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

def remove_dead_code_no_branches(function):
    popped = 0
    used = set()

    i = len(function["instrs"]) - 1

    while i >= 0:
        instruction = function["instrs"][i]
        if ("dest" in instruction and instruction["dest"] not in used) \
        and ("op" not in instruction or instruction["op"] != "call"):
            function["instrs"].pop(i)
            popped += 1
            i -= 1
            continue

        if "dest" in instruction:
            used.remove(instruction["dest"])
        
        if "args" in instruction:
            for arg in instruction["args"]:
                used.add(arg)

        i -= 1

    return popped

if __name__ == "__main__":
    prog = json.load(sys.stdin)
    
    for function in prog["functions"]:
        while remove_dead_code(function):
            pass
    
    json.dump(prog, sys.stdout, indent=2)