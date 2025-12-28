use primitive_types::{U256, U512};
use u256_literal::u256;
use std::collections::HashMap;
use sha3::{Digest, Keccak256};
use chrono::{Utc, Datelike};

const STACK_UFLOW: &str = "Stack underflow";
const MEM_OFLOW: &str = "Memory overflow";
const MSG_SENDER: U256 = u256!(0xDEADBEEFDEADBEEFDEADBEEFDEADBEEF);
const CONTRACT_ADDRESS: U256 = u256!(0xADDDECAFADDDECAFADDDECAFADDDECAF);
const CHAIN_ID: U256 = u256!(0xBEEEEEF);
const BASEFEE: U256 = u256!(1);

#[derive(Debug)]
pub struct ContractAccount {
    pub code: Vec<u8>,
    pub storage: HashMap<U256, U256>,
    pub balance: U256,
}

pub struct Evm<'a> {
    pub pc: usize,
    pub stack: Vec<U256>,
    pub memory: Vec<u8>,
    pub memory_words: usize,
    pub storage: &'a mut HashMap<U256, U256>,
    pub code: &'a [u8],
    pub halted: bool,
    pub calldata: Vec<u8>,
    pub callvalue: U256,
    pub contract_balance: U256,
}

#[derive(Debug)]
pub enum ExitReason {
    Return(Vec<u8>),
    Revert(Vec<u8>),
    Stop,
}

impl ContractAccount {
    pub fn new(code: Vec<u8>) -> Self {
        ContractAccount {
            code: code,
            storage: HashMap::new(),
            balance: U256::zero(),
        }
    }
}

impl<'a> Evm<'a> {
    pub fn new(account: &'a mut ContractAccount, callvalue: U256, calldata: Vec<u8>) -> Self {
        Evm {
            pc: 0,   
            stack: vec![],
            memory: vec![],
            memory_words: 0,
            storage: &mut account.storage,
            code: &account.code,
            halted: false,
            calldata,
            callvalue,
            contract_balance: account.balance,
        }
    }

    pub fn run(&mut self) -> Result<ExitReason, String> {
        while !self.halted {
            let opcode: u8 = self.code[self.pc];
            self.pc += 1;

            match opcode {
                0x00 => return Ok(ExitReason::Stop), // Stop

                0x01 => { // Add
                    let (a, b) = Self::pop_two(self)?;
                    self.stack.push(a + b);
                },

                0x02 => { // Multiply
                    let (a, b) = Self::pop_two(self)?;
                    self.stack.push(a * b);
                },

                0x03 => { // Subtract
                    let (a, b) = Self::pop_two(self)?;
                    self.stack.push(a.overflowing_sub(b).0);
                },

                0x04 => { // Divide
                    let (a, b) = Self::pop_two(self)?;
                    if b.is_zero() {
                        self.stack.push(U256::zero());
                    } else {
                        self.stack.push(a / b);
                    }
                },

                0x05 => { // Signed Divide
                    let (a, b) = Self::pop_two(self)?;
                    if a.is_zero() {
                        self.stack.push(U256::zero());
                    }

                    let sa: i128 = Self::u256_to_i128(a);
                    let sb: i128 = Self::u256_to_i128(b);

                    if sb == i128::MIN && sa == -1 {
                        self.stack.push(b);
                    } else {
                        let result: i128 = sb / sa;
                        self.stack.push(U256::from(result as i128));
                    }
                },

                0x06 => { // MOD
                    let (a, b) = Self::pop_two(self)?;
                    self.stack.push(a % b);
                },

                0x08 => { // ADDMOD
                    let a: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let b: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let n: U256 = self.stack.pop().expect(STACK_UFLOW);
                    if n.is_zero() {
                        self.stack.push(U256::zero());
                    } else {
                        let sum: U256 = a.overflowing_add(b).0;
                        self.stack.push(sum % n);
                    }
                },

                0x09 => { // MULMOD
                    let a: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let b: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let n: U256 = self.stack.pop().expect(STACK_UFLOW);
                    if n.is_zero() {
                        self.stack.push(U256::zero());
                    } else {
                        let prod: U512 = a.full_mul(b);
                        let n512: U512 = U512::from(n);
                        let result: U512 = prod % n512;
                        self.stack.push(Self::u512_to_u256(result));
                    }
                },

                0x0A => { // Exponent
                    let (a, b) = Self::pop_two(self)?;
                    self.stack.push(a.pow(b));
                },

                0x10 => { // LT
                    let (a, b) = Self::pop_two(self)?;
                    self.stack.push(if a < b { U256::one() } else { U256::zero() });
                },

                0x11 => { // GT   
                    let (a, b) = Self::pop_two(self)?;
                    self.stack.push(if a > b { U256::one() } else { U256::zero() });
                },

                0x12 => { // SLT
                    let (a, b) = Self::pop_two(self)?;

                    let a: i128 = Self::u256_to_i128(a);
                    let b: i128 = Self::u256_to_i128(b);

                    self.stack.push(if a < b { U256::one() } else { U256::zero() });
                },

                0x13 => { // SGT
                    let (a, b) = Self::pop_two(self)?;

                    let a: i128 = Self::u256_to_i128(a);
                    let b: i128 = Self::u256_to_i128(b);

                    self.stack.push(if a > b { U256::one() } else { U256::zero() });
                },

                0x14 => { // EQ
                    let (a, b) = Self::pop_two(self)?;
                    self.stack.push(if a == b { U256::one() } else { U256::zero() });
                },

                0x15 => { // ISZERO
                    let a: U256 = self.stack.pop().expect(STACK_UFLOW);
                    self.stack.push(if a.is_zero() { U256::one() } else { U256::zero() });
                },

                0x16 => { // AND
                    let (a, b) = Self::pop_two(self)?;
                    self.stack.push(a & b);
                },

                0x17 => { // OR
                    let (a, b) = Self::pop_two(self)?;
                    self.stack.push(a | b);
                },
                
                0x18 => { // XOR
                    let (a, b) = Self::pop_two(self)?;
                    self.stack.push(a ^ b);
                },

                0x19 => { // NOT
                    let a: U256 = self.stack.pop().expect(STACK_UFLOW);
                    self.stack.push(!a);
                },

                0x1A => { // BYTE
                    let (i, x) = Self::pop_two(self)?;
    
                    let index: usize = i.as_usize();
                    if index >= 32 {
                        self.stack.push(U256::zero());
                    } else {
                        let shift: usize = 8 * (31 - index);
                        let byte: U256 = (x >> shift) & U256::from(0xFF);
                        self.stack.push(byte);
                    }
                },

                0x1B => { // SHL
                    let (shift, value) = Self::pop_two(self)?;
                    self.stack.push(value << shift);
                },

                0x1C => { // SHR
                    let (shift, value) = Self::pop_two(self)?;
                    self.stack.push(value >> shift);
                },

                0x20 => { // KECCAK256
                    let (offset, size) = Self::pop_two(self)?;
                    let hash: U256 = Self::evm_keccak256(self, offset, size)?;
                    self.stack.push(hash);
                },

                0x30 => { // ADDRESS
                    self.stack.push(CONTRACT_ADDRESS);
                },

                0x32 => { // ORIGIN
                    self.stack.push(MSG_SENDER);
                },

                0x33 => { // CALLER
                    self.stack.push(MSG_SENDER);
                },

                0x34 => { // CALLVALUE
                    self.stack.push(self.callvalue);
                },

                0x35 => { // CALLDATALOAD
                    let offset: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let offset: usize = Self::u256_to_usize(offset)?;
                    let mut buf: [u8; 32] = [0u8; 32];

                    for j in 0..32 {
                        if offset + j < self.calldata.len() {
                            buf[j] = self.calldata[offset + j];
                        } else {
                            buf[j] = 0;
                        }
                    }

                    let value: U256 = U256::from_big_endian(&buf);
                    self.stack.push(value);
                },

                0x36 => { // CALLDATASIZE
                    self.stack.push(U256::from(self.calldata.len()));
                },

                0x39 => { // CODECOPY
                    let dest_offset: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let offset: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let size: U256 = self.stack.pop().expect(STACK_UFLOW);

                    let dest_offset: usize = Self::u256_to_usize(dest_offset)?;
                    let offset: usize = Self::u256_to_usize(offset)?;
                    let size: usize = Self::u256_to_usize(size)?;

                    self.check_memory_length(dest_offset + size);

                    for i in 0..size {
                        self.memory[dest_offset + i] = if offset + i < self.code.len() {
                            self.code[offset + i]
                        } else {
                            0u8
                        }
                    }
                },

                0x43 => { // BLOCK NUMBER
                    self.stack.push(U256::from(get_block_num()));
                },

                0x46 => { // CHAINID
                    self.stack.push(CHAIN_ID);
                },

                0x47 => { // SELFBALANCE
                    self.stack.push(self.contract_balance);
                },

                0x48 => { // BASEFEE
                    self.stack.push(BASEFEE);
                },

                0x50 => { // POP
                    self.stack.pop().expect(STACK_UFLOW);
                },

                0x51 => { // MLOAD
                    let offset_u256: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let offset: usize = Self::u256_to_usize(offset_u256)?;

                    let new_size_bytes: usize = offset.checked_add(32).unwrap();
                    let new_size_words: usize = (new_size_bytes + 31) / 32;
                    self.memory_words = self.memory_words.max(new_size_words);

                    let mut buf: [u8; 32] = [0u8; 32];
                    for i in 0..32 {
                        buf[i] = *self.memory.get(offset + i).unwrap_or(&0);
                    }
                    
                    let value: U256 = U256::from_big_endian(&buf);
                    self.stack.push(value);
                },

                0x52 => { // MSTORE
                    let offset_u256: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let value: U256 = self.stack.pop().expect(STACK_UFLOW);

                    let offset: usize = Self::u256_to_usize(offset_u256)?;
                    let end: usize = offset.checked_add(32).expect(MEM_OFLOW);
                    let new_words: usize = (end + 31) / 32;

                    self.memory_words = self.memory_words.max(new_words);

                    self.check_memory_length(end);

                    let mut buf: [u8; 32] = [0u8; 32];
                    value.to_big_endian(&mut buf);

                    for i in 0..32 {
                        self.memory[offset + i] = buf[i];
                    }
                },

                0x53 => { // MSTORE8
                    
                    let offset_u256: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let value: U256 = self.stack.pop().expect(STACK_UFLOW);
                    
                    let offset: usize = Self::u256_to_usize(offset_u256)?;
                    
                    let end: usize = offset.checked_add(1).expect(MEM_OFLOW);
                    let new_words: usize = (end + 31) / 32;
                    self.memory_words = self.memory_words.max(new_words);

                    self.check_memory_length(end);

                    self.memory[offset] = value.low_u32() as u8;
                },

                0x54 => { // SLOAD
                    let key: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let value: U256 = *self.storage.get(&key).unwrap_or(&U256::zero());
                    self.stack.push(value);
                },

                0x55 => { // SSTORE    
                    let key: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let value: U256 = self.stack.pop().expect(STACK_UFLOW);

                    if value.is_zero() {
                        self.storage.remove(&key);
                    } else {
                        self.storage.insert(key, value);
                    };
                },

                0x56 => { // JUMP
                    let counter: usize = Self::u256_to_usize(self.stack.pop().expect(STACK_UFLOW))?;
                    if !self.valid_jumpdest(counter) {
                        return Ok(ExitReason::Revert(vec![0x56]));
                    }
                    self.pc = counter;
                },

                0x57 => { // JUMPI
                    let counter_dest: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let condition: U256 = self.stack.pop().expect(STACK_UFLOW);
                    
                    if condition != U256::zero() {
                        if !self.valid_jumpdest(Self::u256_to_usize(counter_dest)?) {
                            return Ok(ExitReason::Revert(vec![0x57]));
                        }
                        self.pc = Self::u256_to_usize(counter_dest)?
                    };
                },

                0x58 => { // PC
                    self.stack.push(U256::from(self.pc - 1));
                },

                0x5b => { // JUMPDEST
                    // nothing
                },

                0x5f => { // PUSH0
                    self.stack.push(U256::zero());
                },

                0x60..=0x7f => { // PUSHn
                    let n: u8 = opcode - 0x5f;
                    let n: usize = n as usize;

                    if self.pc + n > self.code.len() {
                        return Err("Not enough bytes for PUSH".to_string());
                    }

                    let data: &[u8] = &self.code[self.pc..self.pc + n];
                    self.pc += n;

                    let mut buf = [0u8; 32];
                    buf[32 - data.len()..].copy_from_slice(data);

                    self.stack.push(U256::from_big_endian(&buf));
                },

                0x80..=0x8f => { // DUPn
                    let n: u8 = opcode - 0x7f;
                    let n: usize = n as usize;

                    if self.stack.len() < n {
                        return Err(STACK_UFLOW.to_string());
                    }

                    let value = self.stack[self.stack.len() - n];
                    self.stack.push(value);
                },

                0x90..=0x9f => { // SWAPn
                    let n: usize = (opcode - 0x8f) as usize;
                    let len = self.stack.len();

                    if len <= n {
                        return Err(STACK_UFLOW.to_string());
                    }

                    self.stack.swap(len - 1, len - 1 - n);
                },
                                
                0xf3 => { // RETURN
                    
                    let offset_u256: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let size_u256: U256   = self.stack.pop().expect(STACK_UFLOW);

                    let offset: usize = Self::u256_to_usize(offset_u256)?;
                    let size: usize   = Self::u256_to_usize(size_u256)?;

                    let end: usize = offset.checked_add(size).ok_or(MEM_OFLOW)?;

                    self.check_memory_length(end);

                    let data: Vec<u8> = self.memory[offset..end].to_vec();
                    return Ok(ExitReason::Return(data));
                },

                0xfd => { // REVERT
                    let offset: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let size: U256 = self.stack.pop().expect(STACK_UFLOW);
                    let offset: usize = Self::u256_to_usize(offset)?;
                    let size: usize = Self::u256_to_usize(size)?;
                    
                    let end: usize = offset.checked_add(size).ok_or(MEM_OFLOW)?;
                    self.check_memory_length(end);

                    let data: Vec<u8> = self.memory[offset..end].to_vec();
                    return Ok(ExitReason::Revert(data));
                },

                0xfe => { // INVALID
                    return Ok(ExitReason::Revert(vec![0xfe]));
                },

                _ => return Err(format!("Unknown opcode: {:#x}", opcode)),

            }
        }
        Ok(ExitReason::Stop)
    }

    fn pop_two(&mut self) -> Result<(U256, U256), String>  {
        let a: U256 = self.stack.pop().expect(STACK_UFLOW);
        let b: U256 = self.stack.pop().expect(STACK_UFLOW);
        Ok((a, b))
    }

    fn u256_to_i128(x: U256) -> i128 {
        if x.bit(255) {
            let magnitude: U256 = (!x).overflowing_add(U256::one()).0;
            -(magnitude.low_u128() as i128)
        } else {
            x.low_u128() as i128
        }
    }

    fn u512_to_u256(x: U512) -> U256 {
        let mut buf: [u8; 64] = [0u8; 64];
        x.to_big_endian(&mut buf);
        U256::from_big_endian(&buf[32..])
    }

    fn evm_keccak256(&mut self, offset: U256, size: U256) -> Result<U256, String> {
        let offset: usize = Self::u256_to_usize(offset)?;
        let size: usize = Self::u256_to_usize(size)?;

        let end: usize = offset.checked_add(size).ok_or(MEM_OFLOW)?;

        self.check_memory_length(end);

        let data: &[u8] = &self.memory[offset..end];

        let mut hasher = Keccak256::new();
        hasher.update(data);
        let hash = hasher.finalize();

        Ok(U256::from_big_endian(&hash))
    }

    fn u256_to_usize(x: U256) -> Result<usize, String> {
        if x.bits() > usize::BITS as usize {
            Err("U256 too large to convert to usize".to_string())
        } else {
            Ok(x.as_usize())
        }
    }

    fn check_memory_length(&mut self, end: usize) {
        if self.memory.len() < end {
            self.memory.resize(end, 0u8);
        }
    }

    fn valid_jumpdest(&self, dest: usize) -> bool {
        dest < self.code.len() && self.code[dest] == 0x5b
    }
}

fn get_block_num() -> u32 {
    Utc::now().day() // use day for block num for simplicity
}