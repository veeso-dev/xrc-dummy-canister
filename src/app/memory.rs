use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as IcMemoryManager};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, Storable};
use ic_xrc_types::ExchangeRate;

pub const RATES_MEMORY_ID: MemoryId = MemoryId::new(10);

thread_local! {
    /// Memory manager
    pub static MEMORY_MANAGER: IcMemoryManager<DefaultMemoryImpl> = IcMemoryManager::init(DefaultMemoryImpl::default());
}

#[derive(Clone, Debug, PartialEq)]
pub struct StorableRate(pub ExchangeRate);

impl From<ExchangeRate> for StorableRate {
    fn from(value: ExchangeRate) -> Self {
        Self(value)
    }
}

impl Storable for StorableRate {
    const BOUND: Bound = Bound::Bounded {
        max_size: 4096,
        is_fixed_size: false,
    };

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(&bytes, ExchangeRate).unwrap().into()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Encode!(&self.0).unwrap().into()
    }
}
