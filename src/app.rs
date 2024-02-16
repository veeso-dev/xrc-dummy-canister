//! # App
//!
//! API implementation for deferred canister

mod configuration;
mod memory;

use candid::Nat;
use ic_xrc_types::{ExchangeRateError, GetExchangeRateRequest, GetExchangeRateResult};

use crate::utils::{self};
use crate::InitArgs;

use self::configuration::Configuration;

const XRC_CYCLES_COST: u64 = 10_000_000_000;

pub struct XrcCanister;

impl XrcCanister {
    /// Init fly canister
    pub fn init(data: InitArgs) {
        // Set minting account
        Configuration::set_rates(data.rates);
    }

    pub fn post_upgrade() {}

    /// Returns cycles
    pub fn cycles() -> Nat {
        utils::cycles()
    }

    pub fn get_exchange_rate(request: GetExchangeRateRequest) -> GetExchangeRateResult {
        let cycles = ic_cdk::api::call::msg_cycles_available();
        if cycles < XRC_CYCLES_COST {
            return Err(ExchangeRateError::NotEnoughCycles);
        }
        ic_cdk::api::call::msg_cycles_accept(XRC_CYCLES_COST);
        match Configuration::get_rate(request.base_asset, request.quote_asset) {
            Some(rate) => Ok(rate),
            None => Err(ExchangeRateError::CryptoBaseAssetNotFound),
        }
    }
}
