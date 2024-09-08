
import json
import sys

# Let's not optimize any function that jumps first.

# Meaning focus on purely nicely blocked function

if __name__ == "__main__":
    prog = json.load(sys.stdin)
    json.dump(prog, sys.stdout, indent=2)
