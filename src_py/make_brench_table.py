import sys


if __name__ == "__main__":
    outputs = {}
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        test, passes, status = line.split(",")
        if status == "missing":
            continue
        if test not in outputs:
            outputs[test] = {}
        outputs[test][passes] = status

    PASSES = ["baseline", "dce", "lvn", "dce_lvn"]
    # make MD table
    # make header
    print("| Test |", end="")
    for pass_ in PASSES:
        print(f" {pass_} |", end="")
    print()
    # make separator
    print("|---|", end="")
    for pass_ in PASSES:
        print("---|", end="")
    print()

    for test, results in outputs.items():
        # skip unfinished tests
        if (not results.get("baseline", False)):
            continue
        # skip uninteresting tests
        if (results.get("baseline") <= results.get("dce_lvn")) and ((results.get("lvn") <= results.get("baseline"))):
            continue 
        print(f"| {test} |", end="")
        for pass_ in PASSES:
            print(f" {results.get(pass_, 'N/A')} |", end="")
        print()
