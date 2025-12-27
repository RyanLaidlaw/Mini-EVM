use mini_evm::ContractAccount;

pub fn setup(code: Vec<u8>) -> ContractAccount {
    ContractAccount::new(code)
}