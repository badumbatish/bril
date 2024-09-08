
import json
import sys
from utils import cfg
from bidict import bidict
# Let's not optimize any function that jumps first.


def optimize_blocked_fn(fn):
    # A map of var and instr["op"]
    unused = {}

    for instr in fn["instrs"]:
        if "args" in instr:
            for arg in instr["args"]:
                if arg in unused:
                    del unused[arg]
        if "dest" in instr:
            if instr["dest"]:
                if instr["dest"] in unused:
                    print(f"working {instr["dest"]} : {
                          unused[instr["dest"]]}", file=sys.stderr)
                    unused[instr["dest"]]["op"] = "nop"
                    print(f"working {instr["dest"]} : {
                          unused[instr["dest"]]}", file=sys.stderr)
                unused[instr["dest"]] = instr
    return fn
# Meaning focus on purely nicely blocked function


if __name__ == "__main__":
    prog = json.load(sys.stdin)
    for fn in prog["functions"]:
        if cfg.is_blocked_function(fn):
            optimize_blocked_fn(fn)

    json.dump(prog, sys.stdout, indent=2)
