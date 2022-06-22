use ethereum::{TransactionAction, TransactionV2 as EthereumTransaction};
use ethereum_types::{H160, H256, U256};
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use serde::{Deserialize, Serialize, Serializer};

use std::collections::HashMap;

// Frontier
use fc_rpc_core::types::Bytes;

/// The result type of `txpool` API.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct TxPoolResult<T> {
    pub pending: T,
    pub queued: T,
}

/// The entry maps an origin-address to a batch of scheduled transactions.
/// These batches themselves are maps associating nonces with actual transactions.
pub type TransactionMap<T> = HashMap<H160, HashMap<U256, T>>;

pub trait Get {
    fn get(hash: H256, from: H160, txn: &EthereumTransaction) -> Self;
    fn nonce(txn: &EthereumTransaction) -> U256;
}

/// The exact details of the transaction currently pending for inclusion in the next block(s).
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    /// Block hash
    #[serde(serialize_with = "block_hash_serialize")]
    pub block_hash: Option<H256>,
    /// Block number
    pub block_number: Option<U256>,
    /// Hash
    pub hash: H256,
    /// Transaction index
    pub transaction_index: Option<U256>,
    /// Sender
    pub from: H160,
    /// Recipient
    #[serde(serialize_with = "to_serialize")]
    pub to: Option<H160>,
    /// Nonce
    pub nonce: U256,
    /// Transferred value
    pub value: U256,
    /// Gas
    pub gas: U256,
    /// Gas price
    pub gas_price: U256,
    /// Data
    pub input: Bytes,
}

fn block_hash_serialize<S>(hash: &Option<H256>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("0x{:x}", hash.unwrap_or_default()))
}

fn to_serialize<S>(hash: &Option<H160>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("0x{:x}", hash.unwrap_or_default()))
}

impl Get for Content {
    fn get(hash: H256, from: H160, txn: &EthereumTransaction) -> Self {
        match txn {
            EthereumTransaction::Legacy(txn) => Self {
                block_hash: None,
                block_number: None,
                hash,
                transaction_index: None,
                from,
                to: match txn.action {
                    TransactionAction::Call(to) => Some(to),
                    _ => None,
                },
                nonce: txn.nonce,
                value: txn.value,
                gas: txn.gas_limit,
                gas_price: txn.gas_price,
                input: Bytes(txn.input.clone()),
            },
            EthereumTransaction::EIP2930(txn) => Self {
                block_hash: None,
                block_number: None,
                hash,
                transaction_index: None,
                from,
                to: match txn.action {
                    TransactionAction::Call(to) => Some(to),
                    _ => None,
                },
                nonce: txn.nonce,
                value: txn.value,
                gas: txn.gas_limit,
                gas_price: txn.gas_price,
                input: Bytes(txn.input.clone()),
            },
            EthereumTransaction::EIP1559(txn) => Self {
                block_hash: None,
                block_number: None,
                hash,
                transaction_index: None,
                from,
                to: match txn.action {
                    TransactionAction::Call(to) => Some(to),
                    _ => None,
                },
                nonce: txn.nonce,
                value: txn.value,
                gas: txn.gas_limit,
                gas_price: txn.max_fee_per_gas,
                input: Bytes(txn.input.clone()),
            },
        }
    }

    fn nonce(txn: &EthereumTransaction) -> U256 {
        match txn {
            EthereumTransaction::Legacy(txn) => txn.nonce,
            EthereumTransaction::EIP2930(txn) => txn.nonce,
            EthereumTransaction::EIP1559(txn) => txn.nonce,
        }
    }
}

/// The textual summary of the transaction currently pending for inclusion in the next block(s).
#[derive(Clone, Debug)]
pub struct Inspect {
    /// Recipient
    pub to: Option<H160>,
    /// Transferred value
    pub value: U256,
    /// Gas
    pub gas: U256,
    /// Gas price
    pub gas_price: U256,
}

impl Serialize for Inspect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let res = format!(
            "0x{:x}: {} wei + {} gas x {} wei",
            self.to.unwrap_or_default(),
            self.value,
            self.gas,
            self.gas_price
        );
        serializer.serialize_str(&res)
    }
}

impl Get for Inspect {
    fn get(_hash: H256, _from: H160, txn: &EthereumTransaction) -> Self {
        match txn {
            EthereumTransaction::Legacy(txn) => Self {
                to: match txn.action {
                    TransactionAction::Call(to) => Some(to),
                    _ => None,
                },
                value: txn.value,
                gas: txn.gas_limit,
                gas_price: txn.gas_price,
            },
            EthereumTransaction::EIP2930(txn) => Self {
                to: match txn.action {
                    TransactionAction::Call(to) => Some(to),
                    _ => None,
                },
                value: txn.value,
                gas: txn.gas_limit,
                gas_price: txn.gas_price,
            },
            EthereumTransaction::EIP1559(txn) => Self {
                to: match txn.action {
                    TransactionAction::Call(to) => Some(to),
                    _ => None,
                },
                value: txn.value,
                gas: txn.gas_limit,
                gas_price: txn.max_fee_per_gas,
            },
        }
    }

    fn nonce(txn: &EthereumTransaction) -> U256 {
        match txn {
            EthereumTransaction::Legacy(txn) => txn.nonce,
            EthereumTransaction::EIP2930(txn) => txn.nonce,
            EthereumTransaction::EIP1559(txn) => txn.nonce,
        }
    }
}

#[rpc(server)]
pub trait TxPoolApi {
    /// The content inspection property can be queried to list the exact details of all the
    /// transactions currently pending for inclusion in the next block(s), as well as the ones that
    /// are being scheduled for future execution only.
    ///
    /// The result is an object with two fields pending and queued. Each of these fields are
    /// associative arrays, in which each entry maps an origin-address to a batch of scheduled
    /// transactions. These batches themselves are maps associating nonces with actual transactions.
    ///
    /// For details, see [txpool_content](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_content)
    #[method(name = "txpool_content")]
    fn content(&self) -> RpcResult<TxPoolResult<TransactionMap<Content>>>;

    /// The inspect inspection property can be queried to list a textual summary of all the
    /// transactions currently pending for inclusion in the next block(s), as well as the ones that
    /// are being scheduled for future execution only. This is a method specifically tailored to
    /// developers to quickly see the transactions in the pool and find any potential issues.
    ///
    ///The result is an object with two fields pending and queued. Each of these fields are
    /// associative arrays, in which each entry maps an origin-address to a batch of scheduled
    /// transactions. These batches themselves are maps associating nonces with transactions
    /// summary strings.
    ///
    /// For details, see [txpool_inspect](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_inspect)
    #[method(name = "txpool_inspect")]
    fn inspect(&self) -> RpcResult<TxPoolResult<TransactionMap<Inspect>>>;

    /// The status inspection property can be queried for the number of transactions currently
    /// pending for inclusion in the next block(s), as well as the ones that are being scheduled
    /// for future execution only.
    ///
    /// The result is an object with two fields pending and queued, each of which is a counter
    /// representing the number of transactions in that particular state.
    ///
    /// For details, see [txpool_status](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_status)
    #[method(name = "txpool_status")]
    fn status(&self) -> RpcResult<TxPoolResult<U256>>;
}
