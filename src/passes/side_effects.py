import json
import sys
from utils import cfg

class SideEffects:
    def __init__(self, functions):
        self.side_effect_dict = {"print" : 1}
        self.functions = functions
    
    #wip
    def get_side_effects(self, function):
        name = function["name"]
        if name in self.side_effect_dict:
            if self.side_effect_dict[name] != 0:
                return self.side_effect_dict[name] == 1
        
        # for instruction in function["instrs"]:
        #     if op in instruction and op == "call":