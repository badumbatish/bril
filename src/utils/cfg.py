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


class LVNTable():
    class ID():
        def __init__(self, counter):
            self.counter = counter

    def __init__(self):
        self.id_counter = 0

        # A table of id, list of symbols(dest) and "instrinsic instruction".

        # An instrinsic instruction is where all the args
        # are replace by IDs references, and dest are ignored
        #
        # , where the symbol and value of ID can be found via our
        # table

        self.inner_table = []

    def add_symbol(self, symbol, instrinsic_instruction):
        found_a_place = False
        for row in self.inner_table:
            if self.is_instructionally_equal(instrinsic_instruction, row[2]):
                row[1].append(symbol)
                found_a_place = True
                break
        if found_a_place:
            return
        else:
            self.id_counter += 1
            self.inner_table.append(
                [ID(self.id_counter), symbol, instrinsic_instruction])
            return

    # Transform : a + b -> Id(1) + Id(2)
    # If the instr is just const, then just omit the dest is good enough for us
    def transform_instrinsic_instruction(self, instruction):
        pass

    def is_instructionally_equal(self, instr_1, instr_2):
        pass


def hello_world():
    pass


def hello():
    pass
