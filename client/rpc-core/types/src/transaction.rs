use ethereum::{AccessList, TransactionAction, TransactionV2 as EthereumTransaction};
use ethereum_types::{H160, H256, H512, U256, U64};
use serde::Serialize;
use sha3::{Digest, Keccak256};

use crate::Bytes;

/// TypedTxId of Transaction
#[repr(u8)]
pub enum TypedTxId {
    EIP1559 = 0x02,
    EIP2930 = 0x01,
    Legacy = 0x00,
}

/// Transaction as parity rpc response
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// transaction type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<U64>,
    /// Hash
    pub hash: H256,
    /// Nonce
    pub nonce: U256,
    /// Block hash
    pub block_hash: Option<H256>,
    /// Block number
    pub block_number: Option<U256>,
    /// Transaction Index
    pub transaction_index: Option<U256>,
    /// Sender
    pub from: H160,
    /// Recipient
    pub to: Option<H160>,
    /// Transfered value
    pub value: U256,
    /// Gas Price
    pub gas_price: U256,
    /// Max fee per gas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<U256>,
    /// Gas
    pub gas: U256,
    /// Data
    pub input: Bytes,
    /// Creates contract
    pub creates: Option<H160>,
    /// Raw transaction data
    pub raw: Bytes,
    /// Public key of the signer.
    pub public_key: Option<H512>,
    /// The network id of the transaction, if any.
    pub chain_id: Option<U64>,
    /// The standardised V field of the signature (0 or 1). Used by legacy transaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_v: Option<U256>,
    /// The standardised V field of the signature.
    pub v: U256,
    /// The R field of the signature.
    pub r: U256,
    /// The S field of the signature.
    pub s: U256,
    /// Transaction activates at specified block.
    pub condition: Option<TransactionCondition>,
    /// optional access list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<AccessList>,
    /// miner bribe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<U256>,
}

impl From<EthereumTransaction> for Transaction {
    fn from(tx: EthereumTransaction) -> Self {
        let (signer, public_key) = match recover_signer(&tx) {
            Some(p) => (p.0, Some(p.1)),
            None => (H160::default(), None),
        };

        let creates = create_address(&signer, &tx);
        let raw = rlp::encode(&tx).to_vec();

        match tx {
            EthereumTransaction::Legacy(tx) => Self {
                transaction_type: Some(U64::from(TypedTxId::Legacy as u8)),
                hash: tx.hash(),
                nonce: tx.nonce,
                block_hash: None,
                block_number: None,
                transaction_index: None,
                from: signer,
                to: match tx.action {
                    TransactionAction::Call(to) => Some(to),
                    _ => None,
                },
                value: tx.value,
                gas_price: tx.gas_price,
                max_fee_per_gas: Some(tx.gas_price),
                gas: tx.gas_limit,
                input: tx.input.into(),
                creates,
                raw: raw.into(),
                public_key,
                chain_id: tx.signature.chain_id().map(U64::from),
                standard_v: Some(tx.signature.standard_v().into()),
                v: U256::from(tx.signature.v()),
                r: U256::from(tx.signature.r().as_bytes()),
                s: U256::from(tx.signature.s().as_bytes()),
                condition: None,
                access_list: None,
                max_priority_fee_per_gas: Some(tx.gas_price),
            },
            EthereumTransaction::EIP2930(tx) => Self {
                transaction_type: Some(U64::from(TypedTxId::EIP2930 as u8)),
                hash: tx.hash(),
                nonce: tx.nonce,
                block_hash: None,
                block_number: None,
                transaction_index: None,
                from: signer,
                to: match tx.action {
                    TransactionAction::Call(to) => Some(to),
                    _ => None,
                },
                value: tx.value,
                gas_price: tx.gas_price,
                max_fee_per_gas: Some(tx.gas_price),
                gas: tx.gas_limit,
                input: tx.input.into(),
                creates,
                raw: raw.into(),
                public_key,
                chain_id: Some(tx.chain_id.into()),
                standard_v: None,
                v: U256::from(tx.odd_y_parity as u8),
                r: U256::from(tx.r.as_bytes()),
                s: U256::from(tx.s.as_bytes()),
                condition: None,
                access_list: Some(tx.access_list),
                max_priority_fee_per_gas: Some(tx.gas_price),
            },
            EthereumTransaction::EIP1559(tx) => Self {
                transaction_type: Some(U64::from(TypedTxId::EIP1559 as u8)),
                hash: tx.hash(),
                nonce: tx.nonce,
                block_hash: None,
                block_number: None,
                transaction_index: None,
                from: signer,
                to: match tx.action {
                    TransactionAction::Call(to) => Some(to),
                    _ => None,
                },
                value: tx.value,
                // TODO: not add the block base fee
                gas_price: tx.max_priority_fee_per_gas,
                max_fee_per_gas: Some(tx.max_fee_per_gas),
                gas: tx.gas_limit,
                input: tx.input.into(),
                creates,
                raw: raw.into(),
                public_key,
                chain_id: Some(tx.chain_id.into()),
                standard_v: None,
                v: U256::from(tx.odd_y_parity as u8),
                r: U256::from(tx.r.as_bytes()),
                s: U256::from(tx.s.as_bytes()),
                condition: None,
                access_list: Some(tx.access_list),
                max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
            },
        }
    }
}

/// Represents condition on minimum block number or block timestamp.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub enum TransactionCondition {
    /// Valid at this minimum block number.
    #[serde(rename = "block")]
    Number(u64),
    /// Valid at given unix time.
    #[serde(rename = "time")]
    Timestamp(u64),
}

/// recover_signer recover the signer from the transaction
pub fn recover_signer(transaction: &EthereumTransaction) -> Option<(H160, H512)> {
    let mut sig = [0u8; 65];
    let mut msg = [0u8; 32];

    match transaction {
        EthereumTransaction::Legacy(t) => {
            sig[0..32].copy_from_slice(&t.signature.r()[..]);
            sig[32..64].copy_from_slice(&t.signature.s()[..]);
            sig[64] = t.signature.standard_v();
            msg.copy_from_slice(&ethereum::LegacyTransactionMessage::from(t.clone()).hash()[..]);
        },
        EthereumTransaction::EIP2930(t) => {
            sig[0..32].copy_from_slice(&t.r[..]);
            sig[32..64].copy_from_slice(&t.s[..]);
            sig[64] = t.odd_y_parity as u8;
            msg.copy_from_slice(&ethereum::EIP2930TransactionMessage::from(t.clone()).hash()[..]);
        },
        EthereumTransaction::EIP1559(t) => {
            sig[0..32].copy_from_slice(&t.r[..]);
            sig[32..64].copy_from_slice(&t.s[..]);
            sig[64] = t.odd_y_parity as u8;
            msg.copy_from_slice(&ethereum::EIP1559TransactionMessage::from(t.clone()).hash()[..]);
        },
    }

    let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg).ok()?;
    let address = H160::from(H256::from_slice(Keccak256::digest(&pubkey).as_slice()));

    Some((address, pubkey.into()))
}

/// Get the create address from tx.
pub fn create_address(source: &H160, transaction: &EthereumTransaction) -> Option<H160> {
    let nonce = match transaction {
        EthereumTransaction::Legacy(t) => &t.nonce,
        EthereumTransaction::EIP2930(t) => &t.nonce,
        EthereumTransaction::EIP1559(t) => &t.nonce,
    };

    let action = match transaction {
        EthereumTransaction::Legacy(t) => &t.action,
        EthereumTransaction::EIP2930(t) => &t.action,
        EthereumTransaction::EIP1559(t) => &t.action,
    };

    match action {
        TransactionAction::Call(_) => None,
        TransactionAction::Create => {
            // In Current Version, we just support the legacy create
            let mut stream = rlp::RlpStream::new_list(2);
            stream.append(source);
            stream.append(nonce);
            Some(H256::from_slice(Keccak256::digest(&stream.out()).as_slice()).into())
        },
    }
}
