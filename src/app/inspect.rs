//! # Inspect
//!
//! Deferred inspect message handler

use std::time::Duration;

use candid::{Nat, Principal};
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use icrc_ledger_types::icrc2::approve::{ApproveArgs, ApproveError};
use icrc_ledger_types::icrc2::transfer_from::{TransferFromArgs, TransferFromError};

use super::configuration::Configuration;
use crate::constants::ICRC1_TX_TIME_SKID;
use crate::utils::time;

pub struct Inspect;

impl Inspect {
    /// inspect whether transfer update is valid
    pub fn inspect_transfer(args: &TransferArg) -> Result<(), TransferError> {
        let default_fee: Nat = Configuration::get_fee().into();
        let fee = args.fee.clone().unwrap_or(default_fee.clone());
        if fee < default_fee {
            return Err(TransferError::BadFee {
                expected_fee: default_fee,
            });
        }

        // check if the transaction is too old
        let now = Duration::from_nanos(time());
        let tx_created_at =
            Duration::from_nanos(args.created_at_time.unwrap_or(now.as_nanos() as u64));
        if now > tx_created_at && now.saturating_sub(tx_created_at) > ICRC1_TX_TIME_SKID {
            return Err(TransferError::TooOld);
        } else if tx_created_at.saturating_sub(now) > ICRC1_TX_TIME_SKID {
            return Err(TransferError::CreatedInFuture {
                ledger_time: now.as_nanos() as u64,
            });
        }

        // check memo length
        if let Some(memo) = &args.memo {
            if memo.0.len() < 32 || memo.0.len() > 64 {
                return Err(TransferError::GenericError {
                    error_code: Nat::from(1),
                    message: "Invalid memo length. I must have a length between 32 and 64 bytes"
                        .to_string(),
                });
            }
        }

        Ok(())
    }

    /// inspect icrc2 approve arguments
    pub fn inspect_icrc2_approve(
        caller: Principal,
        args: &ApproveArgs,
    ) -> Result<(), ApproveError> {
        let default_fee: Nat = Configuration::get_fee().into();
        if args.spender.owner == caller {
            return Err(ApproveError::GenericError {
                error_code: 0_u64.into(),
                message: "Spender and owner cannot be equal".to_string(),
            });
        }
        if args
            .fee
            .as_ref()
            .map(|fee| fee < &default_fee)
            .unwrap_or(false)
        {
            return Err(ApproveError::BadFee {
                expected_fee: default_fee,
            });
        }
        // check if expired
        if args
            .expires_at
            .as_ref()
            .map(|expiry| expiry < &time())
            .unwrap_or(false)
        {
            return Err(ApproveError::Expired {
                ledger_time: time(),
            });
        }
        // check if too old or in the future
        if let Some(created_at) = args.created_at_time {
            let current_time = Duration::from_nanos(time());
            let created_at = Duration::from_nanos(created_at);

            if created_at > current_time {
                return Err(ApproveError::CreatedInFuture {
                    ledger_time: current_time.as_nanos() as u64,
                });
            }

            if current_time - created_at > Duration::from_secs(300) {
                return Err(ApproveError::TooOld);
            }
        }

        Ok(())
    }

    pub fn inspect_icrc2_transfer_from(args: &TransferFromArgs) -> Result<(), TransferFromError> {
        let default_fee: Nat = Configuration::get_fee().into();
        // check fee
        if args
            .fee
            .as_ref()
            .map(|fee| fee < &default_fee)
            .unwrap_or(false)
        {
            return Err(TransferFromError::BadFee {
                expected_fee: default_fee,
            });
        }

        // check if too old or in the future
        if let Some(created_at) = args.created_at_time {
            let current_time = Duration::from_nanos(time());
            let created_at = Duration::from_nanos(created_at);

            if created_at > current_time {
                return Err(TransferFromError::CreatedInFuture {
                    ledger_time: current_time.as_nanos() as u64,
                });
            }

            if current_time - created_at > Duration::from_secs(300) {
                return Err(TransferFromError::TooOld);
            }
        }

        // check memo length
        if let Some(memo) = &args.memo {
            if memo.0.len() < 32 || memo.0.len() > 64 {
                return Err(TransferFromError::GenericError {
                    error_code: Nat::from(0),
                    message: "Invalid memo length. I must have a length between 32 and 64 bytes"
                        .to_string(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use icrc_ledger_types::icrc1::transfer::Memo;

    use super::*;
    use crate::app::test_utils;

    #[test]
    fn test_should_inspect_transfer() {
        Configuration::set_fee(10_000);
        let args = TransferArg {
            from_subaccount: None,
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: Some((10_000 - 1).into()),
            memo: None,
            created_at_time: None,
        };

        assert!(Inspect::inspect_transfer(&args).is_err());

        let args = TransferArg {
            from_subaccount: None,
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: Some(10_000.into()),
            memo: None,
            created_at_time: None,
        };

        assert!(Inspect::inspect_transfer(&args).is_ok());

        let args = TransferArg {
            from_subaccount: None,
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: None,
            memo: None,
            created_at_time: None,
        };

        assert!(Inspect::inspect_transfer(&args).is_ok());

        let args = TransferArg {
            from_subaccount: None,
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: None,
            memo: Some(Memo::from(vec![0; 31])),
            created_at_time: None,
        };

        assert!(Inspect::inspect_transfer(&args).is_err());

        let args = TransferArg {
            from_subaccount: None,
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: None,
            memo: Some(Memo::from(vec![0; 65])),
            created_at_time: None,
        };

        assert!(Inspect::inspect_transfer(&args).is_err());

        let args = TransferArg {
            from_subaccount: None,
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: None,
            memo: Some(Memo::from(vec![0; 32])),
            created_at_time: None,
        };

        assert!(Inspect::inspect_transfer(&args).is_ok());

        let args = TransferArg {
            from_subaccount: None,
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: None,
            memo: Some(Memo::from(vec![0; 64])),
            created_at_time: None,
        };

        assert!(Inspect::inspect_transfer(&args).is_ok());
    }

    #[test]
    fn test_should_inspect_icrc2_approve() {
        Configuration::set_fee(10_000);
        let caller = Principal::from_text("aaaaa-aa").unwrap();
        let args = ApproveArgs {
            spender: test_utils::alice_account(),
            amount: 100.into(),
            fee: None,
            expires_at: None,
            created_at_time: None,
            memo: None,
            from_subaccount: None,
            expected_allowance: None,
        };

        assert!(Inspect::inspect_icrc2_approve(caller, &args).is_ok());

        let args = ApproveArgs {
            spender: test_utils::alice_account(),
            amount: 100.into(),
            fee: Some((10_000 - 1).into()),
            expires_at: None,
            created_at_time: None,
            memo: None,
            from_subaccount: None,
            expected_allowance: None,
        };

        assert!(Inspect::inspect_icrc2_approve(caller, &args).is_err());

        let args = ApproveArgs {
            spender: test_utils::alice_account(),
            amount: 100.into(),
            fee: None,
            expires_at: None,
            created_at_time: Some(0),
            memo: None,
            from_subaccount: None,
            expected_allowance: None,
        };

        assert!(Inspect::inspect_icrc2_approve(caller, &args).is_err());

        let args = ApproveArgs {
            spender: test_utils::alice_account(),
            amount: 100.into(),
            fee: None,
            expires_at: Some(0),
            created_at_time: None,
            memo: None,
            from_subaccount: None,
            expected_allowance: None,
        };

        assert!(Inspect::inspect_icrc2_approve(caller, &args).is_err());

        let args = ApproveArgs {
            spender: test_utils::alice_account(),
            amount: 100.into(),
            fee: None,
            expires_at: None,
            created_at_time: Some(crate::utils::time() * 2),
            memo: None,
            from_subaccount: None,
            expected_allowance: None,
        };

        assert!(Inspect::inspect_icrc2_approve(caller, &args).is_err());
    }

    #[test]
    fn test_should_inspect_transfer_from() {
        Configuration::set_fee(10_000);
        let args = TransferFromArgs {
            spender_subaccount: None,
            from: test_utils::alice_account(),
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: Some((10_000 - 1).into()),
            memo: None,
            created_at_time: None,
        };

        assert!(Inspect::inspect_icrc2_transfer_from(&args).is_err());

        let args = TransferFromArgs {
            spender_subaccount: None,
            from: test_utils::alice_account(),
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: Some(10_000.into()),
            memo: None,
            created_at_time: None,
        };

        assert!(Inspect::inspect_icrc2_transfer_from(&args).is_ok());

        let args = TransferFromArgs {
            spender_subaccount: None,
            from: test_utils::alice_account(),
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: None,
            memo: None,
            created_at_time: None,
        };

        assert!(Inspect::inspect_icrc2_transfer_from(&args).is_ok());

        let args = TransferFromArgs {
            spender_subaccount: None,
            from: test_utils::alice_account(),
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: None,
            memo: Some(Memo::from(vec![0; 31])),
            created_at_time: None,
        };

        assert!(Inspect::inspect_icrc2_transfer_from(&args).is_err());

        let args = TransferFromArgs {
            spender_subaccount: None,
            from: test_utils::alice_account(),
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: None,
            memo: Some(Memo::from(vec![0; 65])),
            created_at_time: None,
        };

        assert!(Inspect::inspect_icrc2_transfer_from(&args).is_err());

        let args = TransferFromArgs {
            spender_subaccount: None,
            from: test_utils::alice_account(),
            to: test_utils::bob_account(),
            amount: 100.into(),
            fee: None,
            memo: Some(Memo::from(vec![0; 32])),
            created_at_time: None,
        };

        assert!(Inspect::inspect_icrc2_transfer_from(&args).is_ok());
    }
}
