use crate::core_lib::asset::Asset;

use candid::CandidType;

use bincode;
use serde::{Deserialize, Serialize};

use std::borrow::Cow;

use ic_stable_structures::{storable::Bound, Storable};

type Amount = u128;

#[derive(CandidType, Serialize, Default, Deserialize, Clone)]
pub struct VaultDetails {
    pub asset: Asset,
    pub virtual_asset: Asset,
    pub min_amount: Amount,
}

impl Storable for VaultDetails {
    const BOUND: Bound = Bound::Unbounded;
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        bincode::deserialize(bytes.as_ref()).expect("Failed to deserialize VaultDetails")
    }

    fn to_bytes(&self) -> Cow<[u8]> {
        let serialized = bincode::serialize(self).expect("Failed to serialize MarketDetails");
        Cow::Owned(serialized)
    }
}
