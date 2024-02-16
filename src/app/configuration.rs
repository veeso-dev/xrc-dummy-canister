//! # Configuration
//!
//! Canister configuration

use std::cell::RefCell;

use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableVec};
use ic_xrc_types::{Asset, ExchangeRate};

use super::memory::StorableRate;
use crate::app::memory::{MEMORY_MANAGER, RATES_MEMORY_ID};

thread_local! {

    static RATES: RefCell<StableVec<StorableRate, VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(StableVec::new(MEMORY_MANAGER.with(|mm| mm.get(RATES_MEMORY_ID))).unwrap());

}

/// canister configuration
pub struct Configuration;

impl Configuration {
    /// Set rates
    pub fn set_rates(rates: Vec<ExchangeRate>) {
        RATES.with_borrow_mut(|vec| {
            for rate in rates {
                vec.push(&StorableRate(rate)).unwrap();
            }
        });
    }

    /// Get rate, given the base asset and the quote asset
    pub fn get_rate(base_asset: Asset, quote_asset: Asset) -> Option<ExchangeRate> {
        RATES.with_borrow(|rates| {
            rates
                .iter()
                .find(|rate| rate.0.base_asset == base_asset && rate.0.quote_asset == quote_asset)
                .map(|rate| rate.0)
        })
    }
}

#[cfg(test)]
mod test {

    use ic_xrc_types::{AssetClass, ExchangeRateMetadata};
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_should_set_rate() {
        let usd = Asset {
            symbol: "USD".to_string(),
            class: AssetClass::FiatCurrency,
        };
        let icp = Asset {
            symbol: "ICP".to_string(),
            class: AssetClass::Cryptocurrency,
        };
        let rate: ExchangeRate = ExchangeRate {
            base_asset: usd.clone(),
            quote_asset: icp.clone(),
            rate: 813000000,
            timestamp: 0,
            metadata: ExchangeRateMetadata {
                decimals: 8,
                base_asset_num_queried_sources: 0,
                base_asset_num_received_rates: 0,
                quote_asset_num_queried_sources: 0,
                quote_asset_num_received_rates: 0,
                standard_deviation: 0,
                forex_timestamp: None,
            },
        };
        Configuration::set_rates(vec![rate]);

        let rate = Configuration::get_rate(usd.clone(), icp.clone()).unwrap();
        assert_eq!(rate.base_asset, usd);
        assert_eq!(rate.quote_asset, icp);
        assert_eq!(rate.rate, 813000000);

        assert!(
            Configuration::get_rate(icp.clone(), usd.clone()).is_none(),
            "Rate should not exist"
        );
    }
}
