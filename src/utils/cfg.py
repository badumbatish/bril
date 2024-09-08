branch_instructions = ["jmp", "br", "call", "ret"]


def is_blocked_function(json_function_entry):

    # Check that if inst has an op, then no branch inst
    # Check that there is no labels
    for inst in json_function_entry["instrs"]:
        if "op" in inst:
            for br_inst in branch_instructions:
                if br_inst in inst["op"]:
                    return False
        if "label" in inst:
            return False
    return True


def hello_world():
    pass


def hello():
    pass
