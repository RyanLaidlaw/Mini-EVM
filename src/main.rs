use core::panic;
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
        input_types: Vec<String>,
        output_types: Vec<String>,
    },
    #[serde(rename = "exit")]
    Exit,
}

struct EncodedArg {
    head: Vec<u8>,
    tail: Vec<u8>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input: &String = &args[1];
    let deploy_code: Vec<u8> = hex::decode(input).expect("Error decoding compiled contract");

    let mut account: ContractAccount = ContractAccount::new(deploy_code);

    let mut deploy_evm: Evm<'_> = Evm::new(&mut account, U256::zero(), vec![]);
    let exit: ExitReason = deploy_evm.run().expect("Deployment failed");

    match exit {
        ExitReason::Return(runtime_code) => {
            account.code = runtime_code;
        }
        _ => panic!("Deployment failed"),
    }

    let stdin: Stdin = stdin();
    let mut stdout: Stdout = stdout();

    for line  in stdin.lock().lines() {
        let line: String = line.expect("Could not read stdin line");
        let cmd: Command = serde_json::from_str(&line).expect("Could not read command from stdin");

        match cmd {
            Command::Exit => break,

            Command::Call { signature, args, input_types, output_types } => {
                let selector: [u8; 4] = function_selector(&signature);

                let mut calldata: Vec<u8> = Vec::new();
                calldata.extend_from_slice(&selector);
                
                let mut heads = Vec::new();
                let mut tails = Vec::new();

                for (arg, ty) in args.iter().zip(input_types.iter()) {
                    let encoded: EncodedArg = encode_arg(ty, arg);
                    heads.push(encoded.head);
                    tails.push(encoded.tail);
                }

                let head_size: usize = 32 * heads.len();
                let mut current_offset: usize = head_size;

                for i in 0..heads.len() {
                    if !tails[i].is_empty() {
                        let mut offset_buf: Vec<u8> = vec![0u8; 32];
                        U256::from(current_offset).to_big_endian(&mut offset_buf);
                        heads[i] = offset_buf;
                        current_offset += tails[i].len();
                    }
                }

                for h in heads {
                    calldata.extend_from_slice(&h);
                }
                for t in tails {
                    calldata.extend_from_slice(&t);
                }
                
                // spin up a new instance of the EVM for every call
                let mut evm: Evm<'_> = Evm::new(&mut account, U256::zero(), calldata);

                let result: Result<ExitReason, String> = evm.run();

                match result {
                    Ok(exit) => {
                        match exit {
                            ExitReason::Return(ret) => {
                                decode_return(&mut stdout, ret, output_types);
                            }

                            _ => {
                                writeln!(stdout, "{:?}", exit).expect("Error writing Ok to stdout");
                            }
                        }
                    }
                    Err(e) => {
                        writeln!(stdout, "error: {}", e).expect("Error writing Err to stdout");
                    }
                }

                stdout.flush().expect("Error flushing stdout");
            }
        }
    }
}

fn function_selector(signature: &str) -> [u8; 4] {
    let mut keccak: Keccak = Keccak::v256();
    keccak.update(signature.as_bytes());
    let mut hash: [u8; 32] = [0u8; 32];
    keccak.finalize(&mut hash);
    [hash[0], hash[1], hash[2], hash[3]]
}

fn encode_arg(ty: &str, value: &str) -> EncodedArg {
    match ty {
        "uint256" | "uint" => {
            let v: U256 = U256::from_dec_str(value).expect("invalid uint256");
            let mut buf: Vec<u8> = vec![0u8; 32];
            v.to_big_endian(&mut buf);
            EncodedArg { head: buf, tail: vec![] }
        }

        "address" => {
            let addr: &str = value.strip_prefix("0x").unwrap_or(value);
            let raw: Vec<u8> = hex::decode(addr).expect("invalid address");
            assert_eq!(raw.len(), 20); //uint160

            let mut buf: Vec<u8> = vec![0u8; 32];
            buf[12..].copy_from_slice(&raw);
            EncodedArg { head: buf, tail: vec![] }
        }

        "bool" => {
            let v: u8 = match value {
                "true" | "1" => 1u8,
                "false" | "0" => 0u8,
                _ => panic!("invalid bool"),
            };

            let mut buf: Vec<u8> = vec![0u8; 32];
            buf[31] = v;
            EncodedArg { head: buf, tail: vec![] }
        }

        "string" => {
            let bytes: &[u8] = value.as_bytes();

            let mut len_buf: Vec<u8> = vec![0u8; 32];
            U256::from(bytes.len()).to_big_endian(&mut len_buf);

            let mut data: Vec<u8> = bytes.to_vec();
            while data.len() % 32 != 0 {
                data.push(0);
            }

            EncodedArg {
                head: vec![0u8; 32], // offset filled later
                tail: [len_buf, data].concat(),
            }
        }

        _ => panic!("unsupported type: {}", ty),
    }
}

fn decode_return(stdout: &mut Stdout, ret: Vec<u8>, output_types: Vec<String>) {
    if ret.is_empty() {
        writeln!(stdout, "()").unwrap();
        return;
    }

    let mut outputs: Vec<String> = Vec::new();

    for (i, ty) in output_types.iter().enumerate() {
        let head_offset: usize = i * 32;
        let head: &[u8] = &ret[head_offset..head_offset + 32];

        match ty.as_str() {
            "uint256" | "uint" => {
                let v: U256 = U256::from_big_endian(head);
                outputs.push(v.to_string());
            }

            "bool" => {
                outputs.push((head[31] == 1).to_string());
            }

            "address" => {
                let addr: &[u8] = &head[12..32];
                outputs.push(format!("0x{}", hex::encode(addr)));
            }

            "string" => {
                let offset: usize = U256::from_big_endian(head).as_usize();

                let len_bytes: &[u8] = &ret[offset..offset + 32];
                let len: usize = U256::from_big_endian(len_bytes).as_usize();

                let data_start: usize = offset + 32;
                let data: &[u8] = &ret[data_start..data_start + len];

                let s: String = String::from_utf8_lossy(data).to_string();
                outputs.push(format!("\"{}\"", s));
            }

            _ => outputs.push(format!("<unsupported {}>", ty)),
        }
    }

    if outputs.len() == 1 {
        writeln!(stdout, "{}", outputs[0]).expect("Error writing output to stdout");
    } else {
        writeln!(stdout, "({})", outputs.join(", ")).expect("Error writing output to stdout");
    }
}
