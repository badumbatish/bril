import json
import sys

# Let's not optimize any function that jumps first.

# Meaning focus on purely nicely blocked function
def remove_dead_code(function):
    used = set()

    for instruction in function["instrs"]:
        if 
        used |= instruction["args"]
    
    for instruction in function["instrs"]:
        if instruction["dest"] not in used:
            del instruction

if __name__ == "__main__":
    prog = json.load(sys.stdin)
    
    for function in prog["functions"]:
        remove_dead_code(function)
    
    json.dump(prog, sys.stdout, indent=2)