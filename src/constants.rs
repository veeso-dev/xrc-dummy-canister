use std::time::Duration;

/// The ledger will refuse transactions older than this or newer than this
pub const ICRC1_TX_TIME_SKID: Duration = Duration::from_secs(60 * 5);

/// The ledger canister id of the ICP token
#[cfg(target_arch = "wasm32")]
pub const ICP_LEDGER_CANISTER: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
/// The ledger canister id of the CKBTC token
#[cfg(target_arch = "wasm32")]
pub const CKBTC_LEDGER_CANISTER: &str = "mxzaz-hqaaa-aaaar-qaada-cai";

#[cfg(target_family = "wasm")]
pub const SPEND_ALLOWANCE_EXPIRED_ALLOWANCE_TIMER_INTERVAL: Duration =
    Duration::from_secs(60 * 60 * 24 * 7); // 7 days

#[cfg(target_family = "wasm")]
pub const LIQUIDITY_POOL_SWAP_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24); // 1 day
