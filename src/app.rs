//! # App
//!
//! API implementation for deferred canister

mod balance;
mod configuration;
mod inspect;
mod memory;
mod spend_allowance;
#[cfg(test)]
mod test_utils;

use candid::{CandidType, Nat};
use icrc_ledger_types::icrc::generic_metadata_value::MetadataValue;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::{self, transfer as icrc1_transfer};
use icrc_ledger_types::icrc2;
use serde::Deserialize;
use thiserror::Error;

use self::balance::Balance;
use self::configuration::Configuration;
pub use self::inspect::Inspect;
use self::spend_allowance::SpendAllowance;
use crate::utils::{self, caller};
use crate::InitArgs;

pub type CanisterResult<T> = Result<T, CanisterError>;

#[derive(Clone, Debug, Error, CandidType, PartialEq, Eq, Deserialize)]
pub enum CanisterError {
    #[error("allowance error {0}")]
    Allowance(AllowanceError),
    #[error("balance error {0}")]
    Balance(BalanceError),
    #[error("configuration error {0}")]
    Configuration(ConfigurationError),
    #[error("storage error")]
    StorageError,
    #[error("icrc2 transfer error {0:?}")]
    Icrc2Transfer(icrc2::transfer_from::TransferFromError),
    #[error("icrc1 transfer error {0:?}")]
    Icrc1Transfer(icrc1::transfer::TransferError),
}

impl From<icrc2::transfer_from::TransferFromError> for CanisterError {
    fn from(value: icrc2::transfer_from::TransferFromError) -> Self {
        Self::Icrc2Transfer(value)
    }
}

impl From<icrc1::transfer::TransferError> for CanisterError {
    fn from(value: icrc1::transfer::TransferError) -> Self {
        Self::Icrc1Transfer(value)
    }
}

#[derive(Clone, Debug, Error, CandidType, PartialEq, Eq, Deserialize)]
pub enum AllowanceError {
    #[error("allowance not found")]
    AllowanceNotFound,
    #[error("allowance changed")]
    AllowanceChanged,
    #[error("allowance expired")]
    AllowanceExpired,
    #[error("the spender cannot be the caller")]
    BadSpender,
    #[error("the expiration date is in the past")]
    BadExpiration,
    #[error("insufficient funds")]
    InsufficientFunds,
}

#[derive(Clone, Debug, Error, CandidType, PartialEq, Eq, Deserialize)]
pub enum BalanceError {
    #[error("account not found")]
    AccountNotFound,
    #[error("insufficient balance")]
    InsufficientBalance,
}

#[derive(Clone, Debug, Error, CandidType, PartialEq, Eq, Deserialize)]
pub enum ConfigurationError {
    #[error("there must be at least one admin")]
    AdminsCantBeEmpty,
    #[error("the canister admin cannot be anonymous")]
    AnonymousAdmin,
}

pub struct Icrc2Canister;

impl Icrc2Canister {
    /// Init fly canister
    pub fn init(data: InitArgs) {
        // Set minting account
        Configuration::set_minting_account(data.minting_account);
        // set token data
        Configuration::set_decimals(data.decimals);
        Configuration::set_fee(data.fee);
        Configuration::set_name(data.name);
        Configuration::set_symbol(data.symbol);
        Configuration::set_logo(data.logo);
        // init balances
        Balance::init_balances(data.total_supply, data.accounts);
        // set timers
        Self::set_timers();
    }

    pub fn post_upgrade() {
        Self::set_timers();
    }

    /// Set application timers
    fn set_timers() {
        #[cfg(target_family = "wasm")]
        ic_cdk_timers::set_timer_interval(
            crate::constants::SPEND_ALLOWANCE_EXPIRED_ALLOWANCE_TIMER_INTERVAL,
            SpendAllowance::remove_expired_allowance,
        );
    }

    /// Returns cycles
    pub fn cycles() -> Nat {
        utils::cycles()
    }

    pub fn icrc1_name() -> String {
        Configuration::get_name()
    }

    pub fn icrc1_symbol() -> String {
        Configuration::get_symbol()
    }

    pub fn icrc1_decimals() -> u8 {
        Configuration::get_decimals()
    }

    pub fn icrc1_fee() -> Nat {
        Configuration::get_fee().into()
    }

    pub fn icrc1_metadata() -> Vec<(String, MetadataValue)> {
        vec![
            (
                "icrc1:symbol".to_string(),
                MetadataValue::from(Self::icrc1_symbol()),
            ),
            (
                "icrc1:name".to_string(),
                MetadataValue::from(Self::icrc1_name()),
            ),
            (
                "icrc1:decimals".to_string(),
                MetadataValue::from(Nat::from(Self::icrc1_decimals())),
            ),
            (
                "icrc1:fee".to_string(),
                MetadataValue::from(Self::icrc1_fee()),
            ),
            (
                "icrc1:logo".to_string(),
                MetadataValue::from(Configuration::get_logo()),
            ),
        ]
    }

    pub fn icrc1_total_supply() -> Nat {
        Balance::total_supply()
    }

    pub fn icrc1_minting_account() -> Account {
        Configuration::get_minting_account()
    }

    pub fn icrc1_balance_of(account: Account) -> Nat {
        Balance::balance_of(account).unwrap_or_default()
    }

    pub fn icrc1_transfer(
        transfer_args: icrc1_transfer::TransferArg,
    ) -> Result<Nat, icrc1_transfer::TransferError> {
        // get fee and check if fee is at least ICRC1_FEE
        Inspect::inspect_transfer(&transfer_args)?;
        let fee = transfer_args.fee.unwrap_or(Self::icrc1_fee());

        // get from account
        let from_account = Account {
            owner: utils::caller(),
            subaccount: transfer_args.from_subaccount,
        };

        // check if it is a burn
        if transfer_args.to == Self::icrc1_minting_account() {
            Balance::transfer_wno_fees(from_account, transfer_args.to, transfer_args.amount.clone())
        } else {
            // make transfer
            Balance::transfer(
                from_account,
                transfer_args.to,
                transfer_args.amount.clone(),
                fee.clone(),
            )
        }
        .map_err(|err| match err {
            CanisterError::Balance(BalanceError::InsufficientBalance) => {
                icrc1_transfer::TransferError::InsufficientFunds {
                    balance: Self::icrc1_balance_of(from_account),
                }
            }
            _ => icrc1_transfer::TransferError::GenericError {
                error_code: Nat::from(3),
                message: err.to_string(),
            },
        })?;

        Ok(1.into())
    }

    pub fn icrc1_supported_standards() -> Vec<super::TokenExtension> {
        vec![
            super::TokenExtension::icrc1(),
            super::TokenExtension::icrc2(),
        ]
    }

    pub fn icrc2_approve(
        args: icrc2::approve::ApproveArgs,
    ) -> Result<Nat, icrc2::approve::ApproveError> {
        Inspect::inspect_icrc2_approve(caller(), &args)?;

        let caller_account = Account {
            owner: caller(),
            subaccount: args.from_subaccount,
        };

        let current_allowance = SpendAllowance::get_allowance(caller_account, args.spender).0;

        // pay fee
        let fee = args.fee.clone().unwrap_or(Self::icrc1_fee());
        Balance::transfer_wno_fees(caller_account, Configuration::get_minting_account(), fee)
            .map_err(|_| icrc2::approve::ApproveError::InsufficientFunds {
                balance: Self::icrc1_balance_of(caller_account),
            })?;

        // approve spend
        match SpendAllowance::approve_spend(caller(), args) {
            Ok(amount) => Ok(amount),
            Err(CanisterError::Allowance(AllowanceError::AllowanceChanged)) => {
                Err(icrc2::approve::ApproveError::AllowanceChanged { current_allowance })
            }
            Err(CanisterError::Allowance(AllowanceError::BadExpiration)) => {
                Err(icrc2::approve::ApproveError::TooOld)
            }
            Err(err) => Err(icrc2::approve::ApproveError::GenericError {
                error_code: 0.into(),
                message: err.to_string(),
            }),
        }
    }

    pub fn icrc2_transfer_from(
        args: icrc2::transfer_from::TransferFromArgs,
    ) -> Result<Nat, icrc2::transfer_from::TransferFromError> {
        Inspect::inspect_icrc2_transfer_from(&args)?;

        // check if owner has enough balance
        let owner_balance = Self::icrc1_balance_of(args.from);
        if owner_balance < args.amount {
            return Err(icrc2::transfer_from::TransferFromError::InsufficientFunds {
                balance: owner_balance,
            });
        }

        // check if spender has fee
        let spender = Account {
            owner: caller(),
            subaccount: args.spender_subaccount,
        };
        let spender_balance = Self::icrc1_balance_of(spender);
        let fee = args.fee.clone().unwrap_or(Self::icrc1_fee());
        if spender_balance < fee {
            return Err(icrc2::transfer_from::TransferFromError::InsufficientFunds {
                balance: spender_balance,
            });
        }

        // check allowance
        let (allowance, expires_at) = SpendAllowance::get_allowance(args.from, spender);
        if allowance < args.amount {
            return Err(
                icrc2::transfer_from::TransferFromError::InsufficientAllowance { allowance },
            );
        }

        // check if has expired
        if expires_at.is_some() && expires_at.unwrap() < utils::time() {
            return Err(icrc2::transfer_from::TransferFromError::TooOld);
        }

        // spend allowance
        match SpendAllowance::spend_allowance(
            caller(),
            args.from,
            args.amount.clone(),
            args.spender_subaccount,
        ) {
            Ok(()) => Ok(()),
            Err(CanisterError::Allowance(AllowanceError::InsufficientFunds)) => {
                Err(icrc2::transfer_from::TransferFromError::InsufficientAllowance { allowance })
            }
            Err(CanisterError::Allowance(AllowanceError::AllowanceExpired)) => {
                Err(icrc2::transfer_from::TransferFromError::TooOld)
            }
            Err(e) => Err(icrc2::transfer_from::TransferFromError::GenericError {
                error_code: 0.into(),
                message: e.to_string(),
            }),
        }?;

        // pay fee
        Balance::transfer_wno_fees(spender, Configuration::get_minting_account(), fee.clone())
            .map_err(
                |_| icrc2::transfer_from::TransferFromError::InsufficientFunds {
                    balance: Self::icrc1_balance_of(spender),
                },
            )?;

        // transfer from `from` balance to `to` balance
        Balance::transfer_wno_fees(args.from, args.to, args.amount.clone()).map_err(|_| {
            icrc2::transfer_from::TransferFromError::InsufficientFunds {
                balance: Self::icrc1_balance_of(args.from),
            }
        })?;

        // register transaction
        Ok(1.into())
    }

    pub fn icrc2_allowance(args: icrc2::allowance::AllowanceArgs) -> icrc2::allowance::Allowance {
        let (allowance, expires_at) = SpendAllowance::get_allowance(args.account, args.spender);
        icrc2::allowance::Allowance {
            allowance,
            expires_at,
        }
    }
}

#[cfg(test)]
mod test {

    use icrc_ledger_types::icrc1::transfer::TransferArg;
    use icrc_ledger_types::icrc2::allowance::{Allowance, AllowanceArgs};
    use icrc_ledger_types::icrc2::approve::ApproveArgs;
    use icrc_ledger_types::icrc2::transfer_from::TransferFromArgs;
    use pretty_assertions::assert_eq;

    use self::test_utils::minting_account;
    use super::test_utils::{alice_account, bob_account, caller_account, int_to_decimals};
    use super::*;
    use crate::app::test_utils::bob;
    use crate::constants::ICRC1_TX_TIME_SKID;

    const ICRC1_NAME: &str = "dummy";
    /// Token symbol
    const ICRC1_SYMBOL: &str = "DUM";
    /// pico fly
    const ICRC1_DECIMALS: u8 = 12;
    /// Default transfer fee (10.000 picofly)
    const ICRC1_FEE: u64 = 10_000;
    /// Logo
    const ICRC1_LOGO: &str = "";

    #[tokio::test]
    async fn test_should_init_canister() {
        init_canister();

        // init balance
        assert_eq!(
            Balance::balance_of(alice_account()).unwrap(),
            int_to_decimals(50_000)
        );
        assert_eq!(
            Balance::balance_of(bob_account()).unwrap(),
            int_to_decimals(50_000)
        );
        assert_eq!(
            Balance::balance_of(caller_account()).unwrap(),
            int_to_decimals(100_000)
        );
    }

    #[tokio::test]
    async fn test_should_get_name() {
        init_canister();
        assert_eq!(Icrc2Canister::icrc1_name(), ICRC1_NAME);
    }

    #[tokio::test]
    async fn test_should_get_symbol() {
        init_canister();
        assert_eq!(Icrc2Canister::icrc1_symbol(), ICRC1_SYMBOL);
    }

    #[tokio::test]
    async fn test_should_get_decimals() {
        init_canister();
        assert_eq!(Icrc2Canister::icrc1_decimals(), ICRC1_DECIMALS);
    }

    #[tokio::test]
    async fn test_should_get_fee() {
        init_canister();
        assert_eq!(Icrc2Canister::icrc1_fee(), Nat::from(ICRC1_FEE));
    }

    #[tokio::test]
    async fn test_should_get_metadata() {
        init_canister();
        let metadata = Icrc2Canister::icrc1_metadata();
        assert_eq!(metadata.len(), 5);
        assert_eq!(
            metadata.first().unwrap(),
            &(
                "icrc1:symbol".to_string(),
                MetadataValue::from(ICRC1_SYMBOL)
            )
        );
        assert_eq!(
            metadata.get(1).unwrap(),
            &("icrc1:name".to_string(), MetadataValue::from(ICRC1_NAME))
        );
        assert_eq!(
            metadata.get(2).unwrap(),
            &(
                "icrc1:decimals".to_string(),
                MetadataValue::from(Nat::from(ICRC1_DECIMALS))
            )
        );
        assert_eq!(
            metadata.get(3).unwrap(),
            &(
                "icrc1:fee".to_string(),
                MetadataValue::from(Nat::from(ICRC1_FEE))
            )
        );
        assert_eq!(
            metadata.get(4).unwrap(),
            &("icrc1:logo".to_string(), MetadataValue::from(ICRC1_LOGO))
        );
    }

    #[tokio::test]
    async fn test_should_get_total_supply() {
        init_canister();
        assert_eq!(
            Icrc2Canister::icrc1_total_supply(),
            int_to_decimals(8_888_888)
        );
    }

    #[tokio::test]
    async fn test_should_get_minting_account() {
        init_canister();
        assert_eq!(
            Icrc2Canister::icrc1_minting_account(),
            Configuration::get_minting_account()
        );
    }

    #[tokio::test]
    async fn test_should_get_balance_of() {
        init_canister();
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(alice_account()),
            int_to_decimals(50_000)
        );
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(bob_account()),
            int_to_decimals(50_000)
        );
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(caller_account()),
            int_to_decimals(100_000)
        );
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(Account {
                owner: utils::id(),
                subaccount: Some(utils::random_subaccount().await),
            }),
            Nat::from(0)
        );
    }

    #[tokio::test]
    async fn test_should_transfer() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: int_to_decimals(10_000),
            fee: Some(Nat::from(ICRC1_FEE)),
            created_at_time: Some(utils::time()),
            memo: None,
        };
        assert!(Icrc2Canister::icrc1_transfer(transfer_args).is_ok());
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(caller_account()),
            (int_to_decimals(90_000) - ICRC1_FEE)
        );
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(bob_account()),
            int_to_decimals(60_000)
        );
    }

    #[tokio::test]
    async fn test_should_not_transfer_with_bad_time() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: int_to_decimals(10_000),
            fee: Some(Nat::from(ICRC1_FEE)),
            created_at_time: Some(0),
            memo: None,
        };
        assert!(matches!(
            Icrc2Canister::icrc1_transfer(transfer_args).unwrap_err(),
            icrc1_transfer::TransferError::TooOld { .. }
        ));
    }

    #[tokio::test]
    async fn test_should_not_transfer_with_old_time() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: int_to_decimals(10_000),
            fee: Some(Nat::from(ICRC1_FEE)),
            created_at_time: Some(utils::time() - (ICRC1_TX_TIME_SKID.as_nanos() as u64 * 2)),
            memo: None,
        };
        assert!(matches!(
            Icrc2Canister::icrc1_transfer(transfer_args).unwrap_err(),
            icrc1_transfer::TransferError::TooOld { .. }
        ));
    }

    #[tokio::test]
    async fn test_should_not_transfer_with_time_in_future() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: int_to_decimals(10_000),
            fee: Some(Nat::from(ICRC1_FEE)),
            created_at_time: Some(utils::time() + (ICRC1_TX_TIME_SKID.as_nanos() as u64 * 2)),
            memo: None,
        };
        assert!(matches!(
            Icrc2Canister::icrc1_transfer(transfer_args).unwrap_err(),
            icrc1_transfer::TransferError::CreatedInFuture { .. }
        ));
    }

    #[tokio::test]
    async fn test_should_not_transfer_with_bad_fee() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: int_to_decimals(10_000),
            fee: Some(Nat::from(ICRC1_FEE / 2)),
            created_at_time: Some(utils::time()),
            memo: None,
        };

        assert!(matches!(
            Icrc2Canister::icrc1_transfer(transfer_args).unwrap_err(),
            icrc1_transfer::TransferError::BadFee { .. }
        ));
    }

    #[tokio::test]
    async fn test_should_transfer_with_null_fee() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: int_to_decimals(10_000),
            fee: None,
            created_at_time: Some(utils::time()),
            memo: None,
        };
        assert!(Icrc2Canister::icrc1_transfer(transfer_args).is_ok());
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(caller_account()),
            (int_to_decimals(90_000) - ICRC1_FEE)
        );
    }

    #[tokio::test]
    async fn test_should_transfer_with_higher_fee() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: int_to_decimals(10_000),
            fee: Some(Nat::from(ICRC1_FEE * 2)),
            created_at_time: Some(utils::time()),
            memo: None,
        };
        assert!(Icrc2Canister::icrc1_transfer(transfer_args).is_ok());
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(caller_account()),
            (int_to_decimals(90_000) - (ICRC1_FEE * 2))
        );
    }

    #[tokio::test]
    async fn test_should_not_allow_bad_memo() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: int_to_decimals(10_000),
            fee: None,
            created_at_time: Some(utils::time()),
            memo: Some("9888".as_bytes().to_vec().into()),
        };

        assert!(matches!(
            Icrc2Canister::icrc1_transfer(transfer_args).unwrap_err(),
            icrc1_transfer::TransferError::GenericError { .. }
        ));

        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: int_to_decimals(10_000),
            fee: None,
            created_at_time: Some(utils::time()),
            memo: Some("988898889888988898889888988898889888988898889888988898889888988898889888988898889888988898889888".as_bytes().to_vec().into()),
        };

        assert!(matches!(
            Icrc2Canister::icrc1_transfer(transfer_args).unwrap_err(),
            icrc1_transfer::TransferError::GenericError { .. }
        ));
    }

    #[tokio::test]
    async fn test_should_transfer_with_memo() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: bob_account(),
            amount: int_to_decimals(10_000),
            fee: Some(Nat::from(ICRC1_FEE)),
            created_at_time: Some(utils::time()),
            memo: Some(
                "293458234690283506958436839246024563"
                    .to_string()
                    .as_bytes()
                    .to_vec()
                    .into(),
            ),
        };
        assert!(Icrc2Canister::icrc1_transfer(transfer_args).is_ok());
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(caller_account()),
            (int_to_decimals(90_000) - ICRC1_FEE)
        );
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(bob_account()),
            int_to_decimals(60_000)
        );
    }

    #[tokio::test]
    async fn test_should_burn_from_transfer() {
        init_canister();
        let transfer_args = TransferArg {
            from_subaccount: caller_account().subaccount,
            to: Icrc2Canister::icrc1_minting_account(),
            amount: int_to_decimals(10_000),
            fee: None,
            created_at_time: Some(utils::time()),
            memo: None,
        };
        assert!(Icrc2Canister::icrc1_transfer(transfer_args).is_ok());
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(caller_account()),
            int_to_decimals(90_000)
        );
        assert_eq!(
            Icrc2Canister::icrc1_total_supply(),
            int_to_decimals(8_888_888 - 10_000)
        );
    }

    #[tokio::test]
    async fn test_should_get_supported_extensions() {
        init_canister();
        let extensions = Icrc2Canister::icrc1_supported_standards();
        assert_eq!(extensions.len(), 2);
        assert_eq!(
            extensions.first().unwrap().name,
            crate::TokenExtension::icrc1().name
        );
        assert_eq!(
            extensions.get(1).unwrap().name,
            crate::TokenExtension::icrc2().name
        );
    }

    #[tokio::test]
    async fn test_should_approve_spending() {
        init_canister();
        let approval_args = ApproveArgs {
            from_subaccount: caller_account().subaccount,
            spender: bob_account(),
            amount: int_to_decimals(10_000),
            fee: None,
            expires_at: None,
            expected_allowance: None,
            memo: None,
            created_at_time: None,
        };

        assert!(Icrc2Canister::icrc2_approve(approval_args).is_ok());
        // check allowance
        assert_eq!(
            Icrc2Canister::icrc2_allowance(AllowanceArgs {
                account: caller_account(),
                spender: bob_account(),
            }),
            Allowance {
                allowance: int_to_decimals(10_000),
                expires_at: None,
            }
        );
        // check we have paid fee
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(caller_account()),
            int_to_decimals(100_000) - ICRC1_FEE
        );
    }

    #[tokio::test]
    async fn test_should_not_approve_spending_if_we_cannot_pay_fee() {
        init_canister();
        let approval_args = ApproveArgs {
            from_subaccount: caller_account().subaccount,
            spender: bob_account(),
            amount: int_to_decimals(10_000),
            fee: Some(int_to_decimals(110_000)),
            expires_at: None,
            expected_allowance: None,
            memo: None,
            created_at_time: None,
        };

        assert!(Icrc2Canister::icrc2_approve(approval_args).is_err());
    }

    #[tokio::test]
    async fn test_should_spend_approved_amount() {
        init_canister();
        let approval_args = ApproveArgs {
            from_subaccount: bob_account().subaccount,
            spender: caller_account(),
            amount: int_to_decimals(10_000),
            fee: None,
            expires_at: None,
            expected_allowance: None,
            memo: None,
            created_at_time: None,
        };
        assert!(SpendAllowance::approve_spend(bob(), approval_args).is_ok());
        assert_eq!(
            Icrc2Canister::icrc2_allowance(AllowanceArgs {
                account: bob_account(),
                spender: caller_account(),
            }),
            Allowance {
                allowance: int_to_decimals(10_000),
                expires_at: None,
            }
        );

        // spend
        assert!(Icrc2Canister::icrc2_transfer_from(TransferFromArgs {
            spender_subaccount: caller_account().subaccount,
            from: bob_account(),
            to: alice_account(),
            amount: int_to_decimals(10_000),
            fee: None,
            memo: None,
            created_at_time: None,
        })
        .is_ok());
        // verify balance
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(bob_account()),
            int_to_decimals(40_000)
        );
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(alice_account()),
            int_to_decimals(60_000)
        );
        assert_eq!(
            Icrc2Canister::icrc1_balance_of(caller_account()),
            (int_to_decimals(100_000) - ICRC1_FEE)
        );
        // verify allowance
        assert_eq!(
            Icrc2Canister::icrc2_allowance(AllowanceArgs {
                account: bob_account(),
                spender: caller_account(),
            }),
            Allowance {
                allowance: int_to_decimals(0),
                expires_at: None,
            }
        );
    }

    fn init_canister() {
        let data = InitArgs {
            accounts: vec![
                (alice_account(), int_to_decimals(50_000)),
                (bob_account(), int_to_decimals(50_000)),
                (caller_account(), int_to_decimals(100_000)),
            ],
            symbol: ICRC1_SYMBOL.to_string(),
            name: ICRC1_NAME.to_string(),
            decimals: ICRC1_DECIMALS,
            fee: ICRC1_FEE,
            logo: ICRC1_LOGO.to_string(),
            total_supply: int_to_decimals(8_888_888),
            minting_account: minting_account(),
        };
        Icrc2Canister::init(data);
    }
}
