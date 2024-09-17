import json
from copy import deepcopy
import sys
from utils import cfg

from bidict import bidict
# Let's not optimize any function that jumps first.


def cse(fn):
    # A map of var and instr["op"]

    lvn_table = cfg.LVNTable()

    copy_fn = None
    while copy_fn != fn:
        lvn_table = cfg.LVNTable()

        copy_fn = deepcopy(fn)
        for instr in fn["instrs"]:
            if "dest" in instr:
                lvn_table.add_symbol(instr["dest"], instr)

    return fn


def const_fold(fn):
    # A map of var and instr["op"]

    lvn_table = cfg.const_fold()
    copy_fn = None
    while copy_fn != fn:
        lvn_table = cfg.const_fold()

        copy_fn = deepcopy(fn)
        for instr in fn["instrs"]:
            if "dest" in instr:
                lvn_table.add_symbol(instr["dest"], instr)

    return fn


def lvn(fn):
    cse(fn)
    const_fold(fn)

# Meaning focus on purely nicely blocked function


if __name__ == "__main__":
    prog = json.load(sys.stdin)
    for fn in prog["functions"]:
        if cfg.is_blocked_function(fn):
            lvn(fn)

    json.dump(prog, sys.stdout, indent=2)
