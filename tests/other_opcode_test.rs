use mini_evm::{Evm, ContractAccount};
use primitive_types::U256;
mod common;

#[test]
fn test_byte() {
    let code: Vec<u8> = vec![
        0x63, 0x02, 0x01, 0x10, 0x05, // PUSH4 0x00...02011005
        0x60, 0x1e,                  // PUSH1 30
        0x1a,                        // BYTE
        0x64, 0x02, 0x01, 0x10, 0x05, 0x06, // PUSH5 0x00...0201100506
        0x60, 0x1b,                  // PUSH1 27
        0x1a,                        // BYTE
        0x00                         // STOP
    ];

    let mut account: ContractAccount = common::setup(code);
    let mut evm: Evm = Evm::new(&mut account, U256::zero(), vec![]);
    evm.run().unwrap();

    assert_eq!(evm.stack.len(), 2);
    assert_eq!(evm.stack[0], U256::from(0x10));
    assert_eq!(evm.stack[1], U256::from(0x02));
}

#[test]
fn storage() {
    let code: Vec<u8> = vec![0x60, 0x0A, 0x60, 0x01, 0x55, 0x00];
    let mut account: ContractAccount = common::setup(code);
    let mut evm: Evm = Evm::new(&mut account, U256::zero(), vec![]);

    evm.run().unwrap();
    assert_eq!(evm.stack.len(), 0);
    assert_eq!(account.storage[&U256::from(1)], U256::from(10));
}