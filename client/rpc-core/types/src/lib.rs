use ethereum_types::H256;
use serde::Deserialize;

mod bytes;
pub mod deserialize;
pub mod serialization;
mod transaction;

pub use bytes::*;
use deserialize::*;
pub use transaction::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum RequestBlockId {
    Number(#[serde(deserialize_with = "deserialize_u32_0x")] u32),
    Hash(H256),
    Tag(RequestBlockTag),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RequestBlockTag {
    Earliest,
    Latest,
    Pending,
}
