import os
import subprocess
import numpy as np
from matplotlib import pyplot as plt


def find_bril_files(directory):
    """Find all files ending with .bril in the specified directory and its subdirectories."""
    bril_files = []
    for root, _, files in os.walk(directory):
        for file in files:
            if file.endswith(".bril"):
                bril_files.append(os.path.join(root, file))
    return bril_files


def run_command_on_files(bril_files, command_template):
    """Run the specified command on each .bril file."""
    results = []
    for bril_file in bril_files:
        result = 0
        # print(bril_file)
        command = command_template.format(file=bril_file)
        try:
            # print(f"Running command: {command}")
            result = str(subprocess.check_output(command, shell=True, timeout=2))
        except Exception as e:
            print(f"Error while running command on {bril_file}, running for two seconds, stopping it now {e}")
        
        results.append(result)
    return results


if __name__ == "__main__":
    directory_to_search = "./examples/test/"

    all_command = "cat {file} | bril2json | ./target/debug/liveness_analysis | ./target/debug/optimistic_const_prop | bril2txt | wc -l"
    liveness_command = "cat {file} | bril2json | ./target/debug/liveness_analysis | bril2txt | wc -l"
    opt_const_prop_command = "cat {file} | bril2json | ./target/debug/optimistic_const_prop | bril2txt | wc -l"
    pes_const_prop_command = "cat {file} | bril2json | ./target/debug/pessimistic_const_prop | bril2txt | wc -l"
    none_command = "cat {file} | bril2json | bril2txt | wc -l"

    bril_files = find_bril_files(directory_to_search)

    if bril_files:
        # print(f"Found {len(bril_files)} .bril files.")
        for file in bril_files:
            print(file)
        print("1")
        live_res = run_command_on_files(bril_files, liveness_command)
        print("2")
        opt_res = run_command_on_files(bril_files, opt_const_prop_command)
        # pes_res = run_command_on_files(bril_files, pes_const_prop_command)
        print("3")
        all_res = run_command_on_files(bril_files, all_command)
        print("4")
        none_res = run_command_on_files(bril_files, none_command)
    else:
        print("No .bril files found.")
    
    print("Optimizations: None, Liveness, Optimization, All")
    print("Lines of code left:")
    for a, b, c, d in zip(none_res, live_res, opt_res, all_res):
    # print("Optimizations: None, Liveness, Optimization, All")
      #   print("Lines of code left:")
        print(a, b, c, d)
