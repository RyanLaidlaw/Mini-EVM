<h1 align="center"><strong> Mini-EVM</h1>

## Description

This mini Ethereum Virtual Machine (EVM), built in Rust, gives users the ability to interact with contracts that they "deploy" using the Python run.py script.
This project was created as an introduction to the Rust programming language with the added benefit of learning the lowest levels of the EVM. 

## References/Resources
[EVM Codes](https://www.evm.codes/)
[EtherVM](https://ethervm.io/)
[Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf)

## Requirements
[Python3](https://www.python.org/downloads/)
[Foundry](https://getfoundry.sh/introduction/installation/)
[Rust/Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
[Solidity Compiler (solc)](https://docs.soliditylang.org/en/latest/installing-solidity.html)

## How to Run

Example usage:
`python3 run.py -f Counter.sol`
`python3 run.py -f Bank.sol -c MyContract -t test_files`

## Usage
Calling Smart Contracts in the Mini-EVM follows a similar format to Foundry's Cast. For example:
```
$ python3 run.py -f Counter.sol
cmd> increment()
cmd> number()
cmd> setNumber(uint256) 5
```
> [!NOTE] 
> number() uses Solidity's built in getter for public state variables

> [!NOTE] 
> Functions with uint256 arguments (and possible other types in the future) can forgo the parameter type
> For instance: `setNumber(uint256) 5` is equivalent to `setNumber() 5`

### run.py

This script compiles a Solidity contract found at the specified target folder (which defaults to test_files) and passes the runtime bytecode into the Rust program. This runtime bytecode is stored in the ContractAccount where the EVM can access it.
The program is kept alive to allow the user to interact with the contract, using syntax similar to Foundry's Cast.

### main.rs

This file is used to take in the commands from the Python script. It hashes the function that was called into the calldata and passes the calldata into the EVM, which is spun up for each command.

### lib.rs

This file is where the EVM lives. It parses all of the EVM opcodes and modifies the ContractAccounts storage and returns the output of the code run for each command.
