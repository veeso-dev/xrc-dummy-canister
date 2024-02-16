//! # Configuration
//!
//! Canister configuration

use std::cell::RefCell;

use candid::Principal;
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableCell};
use icrc_ledger_types::icrc1::account::Account;

use super::memory::StorableAccount;
use crate::app::memory::{
    DECIMALS_MEMORY_ID, FEE_MEMORY_ID, LOGO_MEMORY_ID, MEMORY_MANAGER, MINTING_ACCOUNT_MEMORY_ID,
    NAME_MEMORY_ID, SYMBOL_MEMORY_ID,
};

thread_local! {
    /// Minting account
    static MINTING_ACCOUNT: RefCell<StableCell<StorableAccount, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(MINTING_ACCOUNT_MEMORY_ID)),
        Account {
            owner: Principal::anonymous(),
            subaccount: None
        }.into()).unwrap()
    );

    static NAME: RefCell<StableCell<String, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(NAME_MEMORY_ID)), "ICRC-2".to_string()).unwrap());

    static SYMBOL: RefCell<StableCell<String, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(SYMBOL_MEMORY_ID)), "ICRC".to_string()).unwrap());

    static DECIMALS: RefCell<StableCell<u8, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(DECIMALS_MEMORY_ID)), 8).unwrap());

    static FEE: RefCell<StableCell<u64, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(FEE_MEMORY_ID)), 0).unwrap());

    static LOGO: RefCell<StableCell<String, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(LOGO_MEMORY_ID)), "".to_string()).unwrap());


}

/// canister configuration
pub struct Configuration;

impl Configuration {
    /// Set minting account
    pub fn set_minting_account(minting_account: Account) {
        MINTING_ACCOUNT.with_borrow_mut(|cell| {
            cell.set(minting_account.into()).unwrap();
        });
    }

    /// Get minting account address
    pub fn get_minting_account() -> Account {
        MINTING_ACCOUNT.with(|ma| ma.borrow().get().0)
    }

    pub fn set_name(name: String) {
        NAME.with_borrow_mut(|cell| {
            cell.set(name).unwrap();
        });
    }

    pub fn get_name() -> String {
        NAME.with(|name| name.borrow().get().clone())
    }

    pub fn set_symbol(symbol: String) {
        SYMBOL.with_borrow_mut(|cell| {
            cell.set(symbol).unwrap();
        });
    }

    pub fn get_symbol() -> String {
        SYMBOL.with(|symbol| symbol.borrow().get().clone())
    }

    pub fn set_decimals(decimals: u8) {
        DECIMALS.with_borrow_mut(|cell| {
            cell.set(decimals).unwrap();
        });
    }

    pub fn get_decimals() -> u8 {
        DECIMALS.with(|decimals| *decimals.borrow().get())
    }

    pub fn set_fee(fee: u64) {
        FEE.with_borrow_mut(|cell| {
            cell.set(fee).unwrap();
        });
    }

    pub fn get_fee() -> u64 {
        FEE.with(|fee| *fee.borrow().get())
    }

    pub fn set_logo(logo: String) {
        LOGO.with_borrow_mut(|cell| {
            cell.set(logo).unwrap();
        });
    }

    pub fn get_logo() -> String {
        LOGO.with(|logo| logo.borrow().get().clone())
    }
}

#[cfg(test)]
mod test {

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::app::test_utils::bob_account;

    #[test]
    fn test_should_set_minting_account() {
        let minting_account = bob_account();
        Configuration::set_minting_account(minting_account);
        assert_eq!(Configuration::get_minting_account(), minting_account);
    }
}
