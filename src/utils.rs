use candid::{Nat, Principal};

/// Returns current time in nanoseconds
pub fn time() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        time.as_nanos() as u64
    }
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::time()
    }
}

/// Returns canister id
pub fn id() -> Principal {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Principal::from_text("lj532-6iaaa-aaaah-qcc7a-cai").unwrap()
    }
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::id()
    }
}

pub fn cycles() -> Nat {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Nat::from(30_000_000_000_u64)
    }
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::api::canister_balance().into()
    }
}

pub fn caller() -> Principal {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Principal::from_text("zrrb4-gyxmq-nx67d-wmbky-k6xyt-byhmw-tr5ct-vsxu4-nuv2g-6rr65-aae")
            .unwrap()
    }
    #[cfg(target_arch = "wasm32")]
    {
        ic_cdk::caller()
    }
}

/// Generates a random subaccount
#[cfg(test)]
pub async fn random_subaccount() -> icrc_ledger_types::icrc1::account::Subaccount {
    #[cfg(test)]
    {
        let random_bytes = rand::random::<[u8; 32]>();
        icrc_ledger_types::icrc1::account::Subaccount::from(random_bytes)
    }
    #[cfg(not(test))]
    {
        let random_bytes = ic_cdk::api::management_canister::main::raw_rand()
            .await
            .unwrap()
            .0;

        let random_bytes: [u8; 32] = random_bytes.try_into().unwrap();
        icrc_ledger_types::icrc1::account::Subaccount::from(random_bytes)
    }
}
