use std::env;
use std::io::{BufRead, Stdin, Stdout, Write, stdin, stdout};
use serde::Deserialize;
use primitive_types::U256;
use mini_evm::{ContractAccount, Evm, ExitReason};
use tiny_keccak::{Hasher, Keccak};

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Command {
    #[serde(rename = "call")]
    Call {
        signature: String,
        args: Vec<String>,
    },
    #[serde(rename = "exit")]
    Exit,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input: &String = &args[1];
    let runtime_code: Vec<u8> = hex::decode(input).unwrap();

    let mut account: ContractAccount = ContractAccount::new(runtime_code);

    let stdin: Stdin = stdin();
    let mut stdout: Stdout = stdout();

    for line  in stdin.lock().lines() {
        let line: String = line.unwrap();
        let cmd: Command = serde_json::from_str(&line).unwrap();

        match cmd {
            Command::Exit => break,

            Command::Call { signature, args } => {
                let normalized: String = normalize_signature(&signature, &args);
                let selector: [u8; 4] = function_selector(&normalized);

                let mut calldata: Vec<u8> = Vec::new();
                calldata.extend_from_slice(&selector);

                for arg in args.iter() {
                    let value: U256 = U256::from_dec_str(arg).expect("invalid uint256 argument");

                    let mut buf: [u8; 32] = [0u8; 32];
                    value.to_big_endian(&mut buf);
                    calldata.extend_from_slice(&buf);
                }
                
                // spin up a new instance of the EVM for every call
                let mut evm: Evm<'_> = Evm::new(&mut account, U256::zero(), calldata);

                let result: Result<ExitReason, String> = evm.run();

                match result {
                    Ok(exit) => {
                        writeln!(stdout, "{:?}", exit).unwrap();
                    }
                    Err(e) => {
                        writeln!(stdout, "error: {}", e).unwrap();
                    }
                }

                stdout.flush().unwrap();
            }
        }
    }
}

fn function_selector(signature: &str) -> [u8; 4] {
    let mut keccak = Keccak::v256();
    keccak.update(signature.as_bytes());
    let mut hash = [0u8; 32];
    keccak.finalize(&mut hash);
    [hash[0], hash[1], hash[2], hash[3]]
}

fn normalize_signature(sig: &str, args: &[String]) -> String {
    if sig.contains("uint") || sig.contains("address") {
        return sig.to_string();
    }

    let name: &str = sig
        .split('(')
        .next()
        .expect("invalid signature");

    let types = vec!["uint256"; args.len()].join(",");

    format!("{name}({types})")
}
