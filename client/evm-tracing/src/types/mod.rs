//! Runtime API allowing to debug/trace Ethereum

use codec::{Decode, Encode};
use ethereum_types::{H160, H256};

pub mod block;
pub mod replay;
pub mod single;

pub(crate) use crate::formatters::{serialization, Bytes};
use serde::Serialize;
use serialization::*;

pub use block::TransactionTrace;

pub const MANUAL_BLOCK_INITIALIZATION_RUNTIME_VERSION: u32 = 159;

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CallResult {
    Output(Bytes),
    // field "error"
    Error(#[serde(serialize_with = "string_serialize")] Vec<u8>),
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum CreateResult {
    Error {
        #[serde(serialize_with = "string_serialize")]
        error: Vec<u8>,
    },
    Success {
        #[serde(rename = "createdContractAddressHash")]
        created_contract_address_hash: H160,
        #[serde(rename = "createdContractCode")]
        created_contract_code: Bytes,
    },
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CallType {
    Call,
    CallCode,
    DelegateCall,
    StaticCall,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Encode, Decode, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CreateType {
    Create,
}

#[derive(Debug)]
pub enum ContextType {
    Call(CallType),
    Create,
}

impl ContextType {
    pub fn from(opcode: Vec<u8>) -> Option<Self> {
        let opcode = match std::str::from_utf8(&opcode[..]) {
            Ok(op) => op.to_uppercase(),
            _ => return None,
        };
        match &opcode[..] {
            "CREATE" | "CREATE2" => Some(ContextType::Create),
            "CALL" => Some(ContextType::Call(CallType::Call)),
            "CALLCODE" => Some(ContextType::Call(CallType::CallCode)),
            "DELEGATECALL" => Some(ContextType::Call(CallType::DelegateCall)),
            "STATICCALL" => Some(ContextType::Call(CallType::StaticCall)),
            _ => None,
        }
    }
}

pub fn convert_memory(memory: Vec<u8>) -> Vec<H256> {
    let size = 32;
    memory
        .chunks(size)
        .map(|c| {
            let mut msg = [0u8; 32];
            let chunk = c.len();
            if chunk < size {
                let left = size - chunk;
                let remainder = vec![0; left];
                msg[0..left].copy_from_slice(&remainder[..]);
                msg[left..size].copy_from_slice(c);
            } else {
                msg[0..size].copy_from_slice(c)
            }
            H256::from_slice(&msg[..])
        })
        .collect()
}
