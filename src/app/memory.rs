use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as IcMemoryManager};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, Storable};
use icrc_ledger_types::icrc1::account::Account;

pub const BALANCES_MEMORY_ID: MemoryId = MemoryId::new(10);
pub const CANISTER_WALLET_ACCOUNT_MEMORY_ID: MemoryId = MemoryId::new(12);
pub const SPEND_ALLOWANCE_MEMORY_ID: MemoryId = MemoryId::new(14);

// Configuration
pub const MINTING_ACCOUNT_MEMORY_ID: MemoryId = MemoryId::new(20);
pub const NAME_MEMORY_ID: MemoryId = MemoryId::new(21);
pub const SYMBOL_MEMORY_ID: MemoryId = MemoryId::new(22);
pub const DECIMALS_MEMORY_ID: MemoryId = MemoryId::new(23);
pub const FEE_MEMORY_ID: MemoryId = MemoryId::new(24);
pub const LOGO_MEMORY_ID: MemoryId = MemoryId::new(25);

thread_local! {
    /// Memory manager
    pub static MEMORY_MANAGER: IcMemoryManager<DefaultMemoryImpl> = IcMemoryManager::init(DefaultMemoryImpl::default());
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct StorableAccount(pub Account);

impl From<Account> for StorableAccount {
    fn from(value: Account) -> Self {
        Self(value)
    }
}

impl Storable for StorableAccount {
    const BOUND: Bound = Bound::Bounded {
        max_size: 128, // principal + 32 bytes of subaccount
        is_fixed_size: false,
    };

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(&bytes, Account).unwrap().into()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Encode!(&self.0).unwrap().into()
    }
}
