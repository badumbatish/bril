
import json
import sys
from utils import cfg
# Let's not optimize any function that jumps first.


def optimize_blocked_fn(fn):
    # A map of var and inst
    unused = {}

    for instr in fn["instrs"]:
        if "op" in instr:
            for arg in instr["op"]["args"]:

    return fn
# Meaning focus on purely nicely blocked function


if __name__ == "__main__":
    prog = json.load(sys.stdin)
    for fn in prog["functions"]:
        if cfg.is_blocked_function(fn):
            optimize_blocked_fn(fn)

    json.dump(prog, sys.stdout, indent=2)
