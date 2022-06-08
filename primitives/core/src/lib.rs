#![cfg_attr(not(feature = "std"), no_std)]

// re-exports
pub use account::{AccountId20, EthereumSignature, EthereumSigner};

use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
};

/// An index to a block.
pub type BlockNumber = u32;

/// An instant or duration in time.
pub type Moment = u64;

/// Balance of an account.
pub type Balance = u128;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Index of a transaction in the chain.
pub type Index = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = EthereumSignature;
pub type AccountPublic = <Signature as Verify>::Signer;
/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
/// The address format for describing accounts.
pub type Address = AccountId;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block opaque extrinsic type as expected by this runtime.
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
