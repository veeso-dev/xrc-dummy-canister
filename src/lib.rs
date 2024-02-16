//! # Fly
//!
//! The fly canister serves a ICRC-2 token called $FLY, which is the reward token for Deferred transactions.
//! It is a deflationary token which ...

mod app;
mod utils;

use candid::{candid_method, CandidType, Nat};
use ic_cdk_macros::{init, post_upgrade, query, update};
use ic_xrc_types::{ExchangeRate, GetExchangeRateRequest, GetExchangeRateResult};
use serde::Deserialize;

use self::app::XrcCanister;

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct InitArgs {
    pub rates: Vec<ExchangeRate>,
}

#[init]
pub fn init(data: InitArgs) {
    XrcCanister::init(data);
}

#[post_upgrade]
pub fn post_upgrade() {
    XrcCanister::post_upgrade();
}

#[query]
#[candid_method(query)]
pub fn cycles() -> Nat {
    XrcCanister::cycles()
}

#[update]
#[candid_method(update)]
pub fn get_exchange_rate(request: GetExchangeRateRequest) -> GetExchangeRateResult {
    XrcCanister::get_exchange_rate(request)
}

#[allow(dead_code)]
fn main() {
    // The line below generates did types and service definition from the
    // methods annotated with `candid_method` above. The definition is then
    // obtained with `__export_service()`.
    candid::export_service!();
    std::print!("{}", __export_service());
}
