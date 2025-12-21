import subprocess
import argparse
import sys
import json
import os

def call(sig: str, *args: str) -> None:
    """
    Writes commands to the EVM, kind of like foundry cast
    
    @param sig: (str) the function name to call in the contract
    @param args: (str) arguments to pass into the function
    """
    cmd = {
        "type": "call",
        "signature": sig,
        "args": list(map(str, args)),
    }
    proc.stdin.write(json.dumps(cmd) + "\n")
    proc.stdin.flush()
    print(proc.stdout.readline())

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="A script to compile a Solidity contract and run it against the mini EVM")

    parser.add_argument("--file", "-f", type=str, help="Solidity file to compile")
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

    result = subprocess.run(f"solc --bin-runtime {target_folder}/{file}", shell=True, check=True, capture_output=True, text=True).stdout
    binary = result.split("part:")[1].strip()
    
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

    try:
        while True:
            txn = input("\ncmd> ").strip()
            if txn in ("exit", "quit"):
                proc.stdin.write(json.dumps({"type": "exit"}) + "\n")
                proc.stdin.flush()
                break

            parts = txn.split()
            call(parts[0], *parts[1:])
    finally:
        proc.terminate()
