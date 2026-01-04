import subprocess
import argparse
import sys
import json
import os
import shutil

OUT_PATH = "./out"

def call(input_types: dict, sig: str, *args: str) -> None:
    """
    Writes commands to the EVM, kind of like foundry cast
    
    @param sig: (str) the function signature to call in the contract (ex: `setNumber(uint256)`)
    @param args: (str) arguments to pass into the function
    """
    if sig not in input_types:
        print(f"Function signature '{sig}' is incorrect.")
        return
    
    cmd = {
        "type": "call",
        "signature": sig,
        "args": list(args),
        "types": input_types[sig]
    }
    proc.stdin.write(json.dumps(cmd) + "\n")
    proc.stdin.flush()
    print(proc.stdout.readline())

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="A script to compile a Solidity contract and run it on the mini EVM")

    parser.add_argument("--file", "-f", type=str, help="Solidity file to compile")
    parser.add_argument("--contract", "-c", type=str, default=None, help="Specific contract in a file to compile.")
    parser.add_argument("--target-folder", "-t", default='test_files', type=str, help="Folder to look for the file")

    args = parser.parse_args()

    file = args.file
    if file is None:
        print("Please provide a file to use")
        sys.exit(1)

    target_folder = args.target_folder

    if not os.path.exists(f"./{target_folder}"):
        print(f"Could not locate directory: ./{target_folder}")
        sys.exit(1)

    contract = args.contract

    if os.path.exists(OUT_PATH):
        shutil.rmtree(OUT_PATH)
    os.makedirs(OUT_PATH)

    result = subprocess.run(f"solc --bin --optimize --overwrite -o out {target_folder}/{file} --combined-json abi", shell=True, check=True, capture_output=True, text=True).stdout

    json_path = f"{OUT_PATH}/combined.json"
    if not os.path.exists(json_path):
        print("Could not find the abi JSON file. Exiting.")
        sys.exit(1)

    def is_bin(file: str):
        return file.endswith(".bin")
    
    binary_files = list(filter(is_bin, os.listdir(OUT_PATH)))

    if len(binary_files) > 1 and contract is None:
        print("Found more than one contract in the test file. Please specify which to use (-c).")
        sys.exit(1)

    bin_files = [f for f in os.listdir(OUT_PATH) if f.endswith(".bin")]

    binary = None
    if contract is not None:
        path = f"{OUT_PATH}/{contract}.bin"
        if not os.path.exists(path):
            print(f"Contract {contract} not found in output")
            sys.exit(1)
        with open(path, 'r') as f:
            binary = f.read().strip()
    else:
        with open(f"{OUT_PATH}/{bin_files[0]}", 'r') as f:
            binary = f.read().strip()

    if binary is None:
        print("Not able to read binary. Exiting.")
        sys.exit(1)

    subprocess.run(
        ["cargo", "build", "--release"],
        cwd="./",
        check=True
    )

    proc = subprocess.Popen(
        ["./target/release/mini-evm", binary],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
    )

    abi = []
    with open(json_path, 'r') as file:
        abi = json.load(file)

    input_types = {}
    contracts = abi["contracts"]
    for contract, _ in contracts.items():
        for method in contracts[contract]["abi"]:
            if method["type"] == "function":
                name = method["name"]
                types = [inp["type"] for inp in method["inputs"]]
                sig = f"{name}({','.join(types)})"
                input_types[sig] = types


    try:
        while True:
            txn = input("\ncmd> ").strip()
            if txn in ("exit", "quit"):
                proc.stdin.write(json.dumps({"type": "exit"}) + "\n")
                proc.stdin.flush()
                break

            parts = txn.split()
            call(input_types, parts[0], *parts[1:])
    except:
        pass # don't care about ctrl-c error
    finally:
        proc.terminate()
