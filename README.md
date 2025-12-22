# Mini-EVM

## Description
This mini Ethereum Virtual Machine (EVM), built in Rust, gives users the ability to interact with contracts that they "deploy" using the Python run.py script.

### run.py
This script compiles a Solidity contract found at the specified target folder (which defaults to test_files) and passes the runtime bytecode into the Rust program. This runtime bytecode is stored in the ContractAccount where the EVM can access it.
The program is kept alive to allow the user to interact with the contract, using syntax similar to Foundry's Cast.

### main.rs
This file is used to take in the commands from the Python script. It hashes the function that was called into the calldata and passes the calldata into the EVM, which is spun up for each command.

### lib.rs
This file is where the EVM lives. It parses all of the EVM opcodes and modifies the ContractAccounts storage and returns the output of the code run for each command.