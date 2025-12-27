use mini_evm::{Evm, ContractAccount};
use primitive_types::U256;
mod common;

#[test]
fn adds_two_numbers() {
    let code: Vec<u8> = vec![0x60,0x02,0x60,0x03,0x01,0x00];
    let mut account: ContractAccount = common::setup(code);
    let mut evm: Evm = Evm::new(&mut account, U256::zero(), vec![]);

    evm.run().unwrap();
    assert_eq!(evm.stack[0], U256::from(5));
}

#[test]
fn subtracts_two_numbers() {
    let code: Vec<u8> = vec![0x60,0x02,0x60,0x03,0x03,0x00];
    let mut account: ContractAccount = common::setup(code);
    let mut evm: Evm = Evm::new(&mut account, U256::zero(), vec![]);

    evm.run().unwrap();
    assert_eq!(evm.stack.len(), 1);
    assert_eq!(evm.stack[0], U256::from(1));
}

#[test]
fn multiplies_two_numbers() {
    let code: Vec<u8> = vec![0x60,0x03,0x60,0x02,0x02,0x00];
    let mut account: ContractAccount = common::setup(code);
    let mut evm: Evm = Evm::new(&mut account, U256::zero(), vec![]);

    evm.run().unwrap();
    assert_eq!(evm.stack.len(), 1);
    assert_eq!(evm.stack[0], U256::from(6));
}

#[test]
fn divides_two_numbers() {
    let code: Vec<u8> = vec![0x60,0x02,0x60,0x06,0x04,0x00];
    let mut account: ContractAccount = common::setup(code);
    let mut evm: Evm = Evm::new(&mut account, U256::zero(), vec![]);

    evm.run().unwrap();
    assert_eq!(evm.stack.len(), 1);
    assert_eq!(evm.stack[0], U256::from(3));
}

#[test]
fn exponentiation() {
    let code: Vec<u8> = vec![0x60,0x02,0x60,0x02,0x0A,0x00];
    let mut account: ContractAccount = common::setup(code);
    let mut evm: Evm = Evm::new(&mut account, U256::zero(), vec![]);

    evm.run().unwrap();
    assert_eq!(evm.stack.len(), 1);
    assert_eq!(evm.stack[0], U256::from(4));
}

#[test]
fn shift_left() {
    let code: Vec<u8> = vec![0x60,0x04,0x60,0x01,0x1b,0x00];
    let mut account: ContractAccount = common::setup(code);
    let mut evm: Evm = Evm::new(&mut account, U256::zero(), vec![]);

    evm.run().unwrap();
    assert_eq!(evm.stack.len(), 1);
    assert_eq!(evm.stack[0], U256::from(8));
}

#[test]
fn lt_gt_eq() {
    // Less than
    let code: Vec<u8> = vec![0x60,0x02,0x60,0x03,0x10,0x00];
    let mut account: ContractAccount = common::setup(code);
    let mut evm: Evm = Evm::new(&mut account, U256::zero(), vec![]);

    evm.run().unwrap();
    assert_eq!(evm.stack.len(), 1);
    assert_eq!(evm.stack[0], U256::from(0));

    // Greater than 
    let code: Vec<u8> = vec![0x60,0x03,0x60,0x02,0x11,0x00];
    let mut account: ContractAccount = common::setup(code);
    let mut evm: Evm = Evm::new(&mut account, U256::zero(), vec![]);

    evm.run().unwrap();
    assert_eq!(evm.stack.len(), 1);
    assert_eq!(evm.stack[0], U256::from(1));

    // Equal
    let code: Vec<u8> = vec![0x60,0x02,0x60,0x02,0x14,0x00];
    let mut account: ContractAccount = common::setup(code);
    let mut evm: Evm = Evm::new(&mut account, U256::zero(), vec![]);

    evm.run().unwrap();
    assert_eq!(evm.stack.len(), 1);
    assert_eq!(evm.stack[0], U256::from(1));
}