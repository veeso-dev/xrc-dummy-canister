//! # Fly
//!
//! The fly canister serves a ICRC-2 token called $FLY, which is the reward token for Deferred transactions.
//! It is a deflationary token which ...

mod app;
mod constants;
mod inspect;
mod utils;

use candid::{candid_method, CandidType, Nat};
use ic_cdk_macros::{init, post_upgrade, query, update};
use icrc_ledger_types::icrc::generic_metadata_value::MetadataValue;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer as icrc1_transfer;
use icrc_ledger_types::icrc1::transfer::TransferArg;
use serde::Deserialize;

use self::app::Icrc2Canister;

#[derive(CandidType, Clone, Debug)]
pub struct TokenExtension {
    pub name: String,
    pub url: String,
}

impl TokenExtension {
    /// Returns extension for icrc-1
    pub fn icrc1() -> Self {
        Self {
            name: "ICRC-1".to_string(),
            url: "https://github.com/dfinity/ICRC-1".to_string(),
        }
    }

    /// Returns extension for icrc-2
    pub fn icrc2() -> Self {
        Self {
            name: "ICRC-2".to_string(),
            url: "https://github.com/dfinity/ICRC-1".to_string(),
        }
    }
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct InitArgs {
    pub accounts: Vec<(Account, Nat)>,
    pub decimals: u8,
    pub fee: u64,
    pub logo: String,
    pub minting_account: Account,
    pub name: String,
    pub symbol: String,
    pub total_supply: Nat,
}

#[init]
pub fn init(data: InitArgs) {
    Icrc2Canister::init(data);
}

#[post_upgrade]
pub fn post_upgrade() {
    Icrc2Canister::post_upgrade();
}

#[query]
#[candid_method(query)]
pub fn cycles() -> Nat {
    Icrc2Canister::cycles()
}

// icrc-1

#[query]
#[candid_method(query)]
pub fn icrc1_name() -> String {
    Icrc2Canister::icrc1_name()
}

#[query]
#[candid_method(query)]
pub fn icrc1_symbol() -> String {
    Icrc2Canister::icrc1_symbol()
}

#[query]
#[candid_method(query)]
pub fn icrc1_decimals() -> u8 {
    Icrc2Canister::icrc1_decimals()
}

#[query]
#[candid_method(query)]
pub fn icrc1_fee() -> Nat {
    Icrc2Canister::icrc1_fee()
}

#[query]
#[candid_method(query)]
pub fn icrc1_metadata() -> Vec<(String, MetadataValue)> {
    Icrc2Canister::icrc1_metadata()
}

#[query]
#[candid_method(query)]
pub fn icrc1_total_supply() -> Nat {
    Icrc2Canister::icrc1_total_supply()
}

#[query]
#[candid_method(query)]
pub fn icrc1_balance_of(account: Account) -> Nat {
    Icrc2Canister::icrc1_balance_of(account)
}

#[update]
#[candid_method(update)]
pub fn icrc1_transfer(transfer_args: TransferArg) -> Result<Nat, icrc1_transfer::TransferError> {
    Icrc2Canister::icrc1_transfer(transfer_args)
}

#[query]
#[candid_method(query)]
pub fn icrc1_supported_standards() -> Vec<TokenExtension> {
    Icrc2Canister::icrc1_supported_standards()
}

#[update]
#[candid_method(update)]
pub fn icrc2_approve(
    args: icrc_ledger_types::icrc2::approve::ApproveArgs,
) -> Result<Nat, icrc_ledger_types::icrc2::approve::ApproveError> {
    Icrc2Canister::icrc2_approve(args)
}

#[update]
#[candid_method(update)]
pub fn icrc2_transfer_from(
    args: icrc_ledger_types::icrc2::transfer_from::TransferFromArgs,
) -> Result<Nat, icrc_ledger_types::icrc2::transfer_from::TransferFromError> {
    Icrc2Canister::icrc2_transfer_from(args)
}

#[query]
#[candid_method(query)]
pub fn icrc2_allowance(
    args: icrc_ledger_types::icrc2::allowance::AllowanceArgs,
) -> icrc_ledger_types::icrc2::allowance::Allowance {
    Icrc2Canister::icrc2_allowance(args)
}

#[allow(dead_code)]
fn main() {
    // The line below generates did types and service definition from the
    // methods annotated with `candid_method` above. The definition is then
    // obtained with `__export_service()`.
    candid::export_service!();
    std::print!("{}", __export_service());
}
