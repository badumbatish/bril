from functools import reduce
import sys
from copy import deepcopy
branch_instructions = ["jmp", "br", "call", "ret"]


def make_basic_blocks(function):
    curr_block = []
    for inst in function["instrs"]:
        if "label" in inst:
            if curr_block:
                yield curr_block
                curr_block = []
            curr_block.append(inst)
        else: # if not label, must be op
            curr_block.append(inst)
            if inst["op"] in branch_instructions:
                yield curr_block
                curr_block = []
    if curr_block:
        yield curr_block

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

        def __str__(self):
            return f"ID({self.counter})"

        def __repr__(self):
            return f"ID({self.counter})"

        def __eq__(self, other):
            return self.counter == other.counter

    def __init__(self):
        self.id_counter = 0

        # A table of id, list of symbols(dest) and "instrinsic instruction".

        # An instrinsic instruction is where all the args
        # are replace by IDs references, and dest are ignored
        #
        # , where the symbol and value of ID can be found via our
        # table
        # TODO: Transform this inner table into a data struct
        self.inner_table = []

    def __str__(self):
        s = ""
        for x in self.inner_table:
            s += str(x) + "\n"
        return s

    def __repr__(self):
        s = ""
        for x in self.inner_table:
            s += str(x) + "\n"
        return s

    def add_symbol(self, symbol, instruction):
        # TODO: Refactor this removal of a row, but keep the id in there
        found_a_place = False
        instrinsic_instruction = self.__transform_into_instrinsic_instruction__(
            instruction)
        # self.eprint()
        # print(f"{instrinsic_instruction}", file=sys.stderr)
        for row in reversed(self.inner_table):

            if self.is_instructionally_equal(instrinsic_instruction, row[2]):
                row[1].append(symbol)
                found_a_place = True
                break
        if found_a_place:
            for row in self.inner_table[:self.id_counter-1]:
                if symbol in row[1]:
                    row[1].remove(symbol)
                    row[2] = {}
            pass
        else:
            self.inner_table.append(
                [self.ID(self.id_counter), [symbol], deepcopy(instrinsic_instruction)])
            for row in self.inner_table[:self.id_counter-1]:
                if symbol in row[1]:
                    row[1].remove(symbol)
                    row[2] = {}
            self.id_counter += 1
        self.__transform_from_instrinsic_instruction__(instrinsic_instruction)

    # Transform : a + b -> Id(1) + Id(2)
    # If the instr is just const, then just omit the dest is good enough for us
    def __transform_into_instrinsic_instruction__(self, instruction):
        # print(f"before transforming into instrinsics:\n {
        #       instruction}", file=sys.stderr)
        if "args" in instruction:
            temp = [self.__query_symbol_to_id__(
                arg) for arg in instruction["args"]]
            instruction["args"] = temp

        # print(f"after transforming into instrinsics:\n {
        #       instruction}\n\n", file=sys.stderr)
        return instruction

    def __transform_from_instrinsic_instruction__(self, instruction):
        # print(f"before transforming from instrinsics:\n {
        #       instruction}", file=sys.stderr)
        stop = False
        if not stop and "dest" in instruction:
            # common sub expression
            for row in reversed(self.inner_table):
                if instruction["dest"] in row[1] and instruction["dest"] != row[1][0]:
                    instruction["op"] = "id"
                    instruction["args"] = [row[1][0]]

                    stop = True
                    break

            # exhaustive: we just replace id, incurring 1 computation or more
            if not stop:
                if "args" in instruction:
                    instruction["args"] = [self.__query_id_to_symbol__(id)
                                           if isinstance(id, self.ID)
                                           else id for id in instruction["args"]]
        # if not stop and "args" in instruction:
        #
        #     instruction["args"] = [self.inner_table[arg.counter][1][0]
        #                            if isinstance(arg, self.ID) and self.inner_table[arg.counter][1][0] != arg else arg
        #                            for arg in instruction["args"]
        #                            ]
        #     stop = True

        # print(f"after transforming from instrinsics:\n {
        #       instruction}", file=sys.stderr)
        return instruction

    def __query_symbol_to_id__(self, symbol):
        for row in reversed(self.inner_table):
            for sym in row[1]:
                if symbol == sym:
                    return row[0]

        return self.ID(-1)

    def __query_id_to_symbol__(self, id):
        return self.inner_table[id.counter][1][0]

    def is_instructionally_equal(self, instr_1, instr_2):
        # print(f"Comparing {instr_1} and {instr_2}", file=sys.stderr)
        args1 = []
        if "args" in instr_1:
            args1 = [arg for arg in instr_1["args"]]

        args2 = []
        if "args" in instr_2:
            args2 = [arg for arg in instr_2["args"] if "args" in instr_2]

        values1 = ("empty", "empty")
        if "value" in instr_1:
            values1 = (instr_1["type"], instr_1["value"])

        values2 = ("empty", "empty")
        if "value" in instr_2:
            values2 = (instr_2["type"], instr_2["value"])
        # print(f"{args1 == args2}", file=sys.stderr)
        return args1 == args2 and values1 == values2 and instr_1["op"] == instr_2["op"]

    def eprint(self):
        print(f"{self}", file=sys.stderr)


class const_fold(LVNTable):

    def __init__(self):
        super().__init__()

    def __transform_from_instrinsic_instruction__(self, instruction):
        # print(f"before transforming from instrinsics:\n {
        #       instruction", file=sys.stderr)
        stop = False
        if not stop and "dest" in instruction:
            # common sub expression

            arg_values = []
            if "args" in instruction:
                for arg in instruction["args"]:
                    arg_value = self.__is_arg_const(
                        arg, instruction["type"] if "type" in instruction else None)
                    if arg_value is not None:
                        arg_values.append(arg_value)
                potential_candidate = ["add", "sub", "mul", "div"]
                if len(arg_values) == len(instruction["args"]) and (instruction["op"] in potential_candidate):
                    if instruction["op"] == "add":
                        instruction["value"] = reduce(
                            lambda x, y: x + y, arg_values)
                        del instruction["args"]
                        instruction["op"] = "const"
                    if instruction["op"] == "sub":
                        instruction["value"] = reduce(
                            lambda x, y: x - y, arg_values)
                        del instruction["args"]
                        instruction["op"] = "const"

                    if instruction["op"] == "mul":
                        instruction["value"] = reduce(
                            lambda x, y: x * y, arg_values)
                        del instruction["args"]
                        instruction["op"] = "const"

                    if instruction["op"] == "div":
                        instruction["value"] = reduce(
                            lambda x, y: x / y, arg_values)
                        del instruction["args"]
                        instruction["op"] = "const"
                    stop = True
            # exhaustive: we just replace id, incurring 1 computation or more
            if not stop:
                if "args" in instruction:
                    instruction["args"] = [self.__query_id_to_symbol__(id)
                                           if isinstance(id, self.ID)
                                           else id for id in instruction["args"]]
        # if not stop and "args" in instruction:
        #
        #     instruction["args"] = [self.inner_table[arg.counter][1][0]
        #                            if isinstance(arg, self.ID) and self.inner_table[arg.counter][1][0] != arg else arg
        #                            for arg in instruction["args"]
        #                            ]
        #     stop = True

        # print(f"after transforming from instrinsics:\n {
        #       instruction}", file=sys.stderr)
        return instruction

    def __is_arg_const(self, arg, type="int"):
        # print(arg, file=sys.stderr)
        for row in reversed(self.inner_table):
            if arg == row[0] and "op" in row[2] and "const" in row[2]["op"]:
                if type == "int":
                    return int(row[2]["value"])
                elif type == "float":
                    return float(row[2]["value"])

        return None


def hello_world():
    pass


def hello():
    pass
