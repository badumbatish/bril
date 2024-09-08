import json
import sys

# Let's not optimize any function that jumps first.

# Meaning focus on purely nicely blocked function
def remove_dead_code(function):
    used = set()

    for instruction in function["instrs"]:
        if "args" in instruction:
            for arg in instruction["args"]:
                used.add(arg)
    print(used)
    for instruction in function["instrs"]:
        print(instruction)
        if "dest" in instruction and instruction["dest"] not in used:
            print instruction["dest"]
            if "op" not in instruction or instruction["op"] != "call":
                del instruction

if __name__ == "__main__":
    prog = json.load(sys.stdin)
    
    for function in prog["functions"]:
        remove_dead_code(function)
    
    #json.dump(prog, sys.stdout, indent=2)