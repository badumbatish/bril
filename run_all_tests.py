import os
import subprocess

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
    for bril_file in bril_files:
        command = command_template.format(file=bril_file)
        try:
            #print(f"Running command: {command}")
            subprocess.run(command, shell=True, check=True)
        except subprocess.CalledProcessError as e:
            print(f"Error while running command on {bril_file}: {e}")

if __name__ == "__main__":
    directory_to_search = "./benchmarks/"
    
    all_command = "cat {file} | bril2json | ./target/debug/liveness_analysis | ./target/debug/optimistic_const_prop | ./target/debug/pessimistic_const_prop | bril2txt | wc -l"
    liveness_command = "cat {file} | bril2json | ./target/debug/liveness_analysis | bril2txt | wc -l"
    opt_const_prop_command = "cat {file} | bril2json | ./target/debug/optimistic_const_prop | bril2txt | wc -l"
    pes_const_prop_command = "cat {file} | bril2json | ./target/debug/pessimistic_const_prop | bril2txt | wc -l"

    bril_files = find_bril_files(directory_to_search)

    if bril_files:
        #print(f"Found {len(bril_files)} .bril files.")
        run_command_on_files(bril_files, opt_const_prop_command)
    else:
        print("No .bril files found.")
