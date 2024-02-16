//! # Balances
//!
//! ICRC-1 token balances

mod account_balance;

use std::cell::RefCell;

use candid::{Nat, Principal};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use icrc_ledger_types::icrc1::account::Account;
use num_bigint::BigUint;

use self::account_balance::Balance as AccountBalance;
use super::configuration::Configuration;
use super::{BalanceError, CanisterError, CanisterResult};
use crate::app::memory::{
    StorableAccount, BALANCES_MEMORY_ID, CANISTER_WALLET_ACCOUNT_MEMORY_ID, MEMORY_MANAGER,
};

thread_local! {
    /// Account balances
    static BALANCES: RefCell<StableBTreeMap<StorableAccount, AccountBalance, VirtualMemory<DefaultMemoryImpl>>> =
        RefCell::new(StableBTreeMap::new(MEMORY_MANAGER.with(|mm| mm.get(BALANCES_MEMORY_ID)))
    );

    /// Wallet which contains all the native tokens of the canister
    static CANISTER_WALLET_ACCOUNT: RefCell<StableCell<StorableAccount, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableCell::new(MEMORY_MANAGER.with(|mm| mm.get(CANISTER_WALLET_ACCOUNT_MEMORY_ID)),
        Account {
        owner: Principal::anonymous(),
        subaccount: None,
    }.into()).unwrap());
}

pub struct Balance;

impl Balance {
    /// Set init balances
    ///
    /// WARNING: this function DOESN'T check anything and it's meant to be used only on init.
    /// Panics if initializing more than total supply.
    pub fn init_balances(total_supply: Nat, initial_balances: Vec<(Account, Nat)>) {
        // make canister acount
        let canister_account = Account {
            owner: crate::utils::id(),
            subaccount: None,
        };
        // set canister
        CANISTER_WALLET_ACCOUNT.with_borrow_mut(|wallet| {
            wallet
                .set(StorableAccount::from(canister_account))
                .expect("failed to set canister account");
        });

        BALANCES.with_borrow_mut(|balances| {
            let canister_balance =
                Nat(total_supply.0 - initial_balances.iter().map(|(_, b)| &b.0).sum::<BigUint>());
            // init accounts
            for (account, balance) in initial_balances {
                let storable_account = StorableAccount::from(account);
                balances.insert(storable_account, balance.clone().into());
            }
            // set remaining supply to canister account
            balances.insert(
                StorableAccount::from(canister_account),
                AccountBalance {
                    amount: canister_balance,
                },
            );
        });
    }

    pub fn total_supply() -> Nat {
        let minting_account = Configuration::get_minting_account();
        BALANCES.with_borrow(|balances| {
            let mut supply = Nat::from(0);
            for (account, balance) in balances.iter() {
                if minting_account != account.0 {
                    supply += balance.amount;
                }
            }

            supply
        })
    }

    /// Get balance of account
    pub fn balance_of(account: Account) -> CanisterResult<Nat> {
        Self::with_balance(account, |balance| balance.amount.clone())
    }

    /// Transfer $picoFly tokens from `from` account to `to` account.
    /// The fee is transferred to the Minting Account, making it burned
    pub fn transfer(from: Account, to: Account, value: Nat, fee: Nat) -> CanisterResult<()> {
        // verify balance
        let to_spend = value.clone() + fee.clone();
        if Self::balance_of(from)? < to_spend {
            return Err(CanisterError::Balance(BalanceError::InsufficientBalance));
        }

        // transfer without fees from -> to
        Self::transfer_wno_fees(from, to, value)?;

        // then pay fees
        if fee > 0_u64 {
            Self::transfer_wno_fees(from, Configuration::get_minting_account(), fee)
        } else {
            Ok(())
        }
    }

    /// Transfer $picoFly tokens from canister to `to` account.
    ///
    /// This function is meant to be used only by the deferred canister and does not apply fees or burns.
    pub fn transfer_wno_fees(from: Account, to: Account, value: Nat) -> CanisterResult<()> {
        Self::with_balance_mut(from, |balance| {
            if balance.amount < value {
                return Err(CanisterError::Balance(BalanceError::InsufficientBalance));
            }
            balance.amount -= value.clone();
            Ok(())
        })?;
        Self::with_balance_mut(to, |balance| {
            balance.amount += value;
            Ok(())
        })
    }

    fn with_balance<F, T>(account: Account, f: F) -> CanisterResult<T>
    where
        F: FnOnce(&AccountBalance) -> T,
    {
        let storable_account = StorableAccount::from(account);
        BALANCES.with_borrow(|balances| match balances.get(&storable_account) {
            Some(balance) => Ok(f(&balance)),
            None => Err(CanisterError::Balance(BalanceError::AccountNotFound)),
        })
    }

    fn with_balance_mut<F, T>(account: Account, f: F) -> CanisterResult<T>
    where
        F: FnOnce(&mut AccountBalance) -> CanisterResult<T>,
    {
        let storable_account = StorableAccount::from(account);
        BALANCES.with_borrow_mut(|balances| {
            let mut balance = match balances.get(&storable_account) {
                Some(balance) => balance,
                None => {
                    // If balance is not set, create it with 0 balance
                    balances.insert(storable_account.clone(), AccountBalance::from(Nat::from(0)));
                    balances.get(&storable_account).unwrap()
                }
            };
            let res = f(&mut balance)?;

            balances.insert(storable_account, balance);

            Ok(res)
        })
    }
}

#[cfg(test)]
mod test {

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::app::test_utils::{alice_account, bob_account, int_to_decimals};
    use crate::utils::{self};

    #[test]
    fn test_should_init_balances() {
        let total_supply = int_to_decimals(8_888_888);

        let initial_balances = vec![
            (alice_account(), int_to_decimals(188_888)),
            (bob_account(), int_to_decimals(100_000)),
        ];

        Balance::init_balances(total_supply, initial_balances);

        let canister_account = CANISTER_WALLET_ACCOUNT.with_borrow(|wallet| wallet.get().0);
        assert_eq!(
            Balance::balance_of(canister_account).unwrap(),
            int_to_decimals(8_888_888 - 188_888 - 100_000)
        );

        assert_eq!(
            Balance::balance_of(alice_account()).unwrap(),
            int_to_decimals(188_888)
        );
        assert_eq!(
            Balance::balance_of(bob_account()).unwrap(),
            int_to_decimals(100_000)
        );
    }

    #[tokio::test]
    async fn test_should_transfer_from_canister() {
        let total_supply = int_to_decimals(8_888_888);
        let recipient_account = Account {
            owner: utils::id(),
            subaccount: Some(utils::random_subaccount().await),
        };

        let initial_balances = vec![(recipient_account, int_to_decimals(888))];

        Balance::init_balances(total_supply, initial_balances);

        assert_eq!(
            Balance::balance_of(recipient_account).unwrap(),
            int_to_decimals(888)
        );
    }

    #[test]
    fn test_should_transfer_between_accounts() {
        let total_supply = int_to_decimals(8_888_888);
        let initial_balances = vec![
            (alice_account(), int_to_decimals(120)),
            (bob_account(), int_to_decimals(50)),
        ];
        Balance::init_balances(total_supply, initial_balances);

        // transfer
        assert!(Balance::transfer(
            alice_account(),
            bob_account(),
            int_to_decimals(50),
            int_to_decimals(1)
        )
        .is_ok());
        // verify balances
        assert_eq!(
            Balance::balance_of(alice_account()).unwrap(),
            int_to_decimals(120 - 50 - 1)
        );
        assert_eq!(
            Balance::balance_of(bob_account()).unwrap(),
            int_to_decimals(100)
        );
        // fee should be burned
        assert_eq!(Balance::total_supply(), int_to_decimals(8_888_888 - 1));
    }

    #[test]
    fn test_should_fail_transfer_if_has_no_balance_to_pay_fee() {
        let total_supply = int_to_decimals(8_888_888);
        let initial_balances = vec![
            (alice_account(), int_to_decimals(50)),
            (bob_account(), int_to_decimals(50)),
        ];
        Balance::init_balances(total_supply, initial_balances);

        // transfer
        assert!(Balance::transfer(
            alice_account(),
            bob_account(),
            int_to_decimals(50),
            int_to_decimals(1)
        )
        .is_err());
    }

    #[test]
    fn test_should_not_pay_fee_if_fee_is_zero() {
        let total_supply = int_to_decimals(8_888_888);
        let initial_balances = vec![
            (alice_account(), int_to_decimals(50)),
            (bob_account(), int_to_decimals(50)),
        ];
        Balance::init_balances(total_supply, initial_balances);

        // transfer
        assert!(Balance::transfer(
            alice_account(),
            bob_account(),
            int_to_decimals(50),
            int_to_decimals(0)
        )
        .is_ok());
        // verify balances
        assert_eq!(
            Balance::balance_of(alice_account()).unwrap(),
            int_to_decimals(0)
        );
        assert_eq!(
            Balance::balance_of(bob_account()).unwrap(),
            int_to_decimals(100)
        );
        // fee should be burned
        assert_eq!(Balance::total_supply(), int_to_decimals(8_888_888));
    }

    #[test]
    fn test_should_not_allow_transfer_if_not_enough_balance() {
        let total_supply = int_to_decimals(8_888_888);
        let initial_balances = vec![
            (alice_account(), int_to_decimals(50)),
            (bob_account(), int_to_decimals(50)),
        ];
        Balance::init_balances(total_supply, initial_balances);

        // transfer
        assert!(Balance::transfer(
            alice_account(),
            bob_account(),
            int_to_decimals(100),
            int_to_decimals(1)
        )
        .is_err());
    }

    #[test]
    fn test_should_get_total_supply() {
        let total_supply = int_to_decimals(8_888_888);
        let initial_balances = vec![(bob_account(), int_to_decimals(100_000))];
        Balance::init_balances(total_supply, initial_balances);
        assert_eq!(Balance::total_supply(), int_to_decimals(8_888_888));

        // burn
        assert!(Balance::transfer_wno_fees(
            bob_account(),
            Configuration::get_minting_account(),
            int_to_decimals(100_000)
        )
        .is_ok());
        assert_eq!(
            Balance::total_supply(),
            int_to_decimals(8_888_888 - 100_000)
        );
    }
}
