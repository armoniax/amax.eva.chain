#![cfg_attr(not(feature = "std"), no_std)]

pub mod ethereum;
pub mod evm_config;
pub mod pallets;
pub mod precompiles;

// Substrate
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_finality_grandpa::AuthorityId as GrandpaId;
// Frontier
pub use pallet_ethereum::Transaction as EthereumTransaction;
// Local
pub use runtime_common_constants as constants;

use frame_support::traits::{Currency, FindAuthor, OnUnbalanced};
use primitives_core::{AccountId, AccountId20};
use sp_core::H160;
use sp_runtime::ConsensusEngineId;
use sp_std::marker::PhantomData;

type NegativeImbalance<B> = <B as Currency<AccountId>>::NegativeImbalance;

pub struct ToAuthor<Runtime, B>(PhantomData<(Runtime, B)>);
impl<Runtime, B> OnUnbalanced<NegativeImbalance<B>> for ToAuthor<Runtime, B>
where
    Runtime: pallet_authorship::Config + frame_system::Config<AccountId = AccountId>,
    B: Currency<AccountId>,
{
    fn on_nonzero_unbalanced(amount: NegativeImbalance<B>) {
        if let Some(author) = pallet_authorship::Pallet::<Runtime>::author() {
            <B as Currency<AccountId>>::resolve_creating(&author, amount);
        }
    }
}

pub struct CoinbaseAuthor<Runtime, F>(PhantomData<(Runtime, F)>);
impl<Runtime, F> FindAuthor<AccountId20> for CoinbaseAuthor<Runtime, F>
where
    Runtime: pallet_session::Config<ValidatorId = AccountId20>,
    F: FindAuthor<u32>,
{
    fn find_author<'a, I>(_digests: I) -> Option<AccountId20>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        #[cfg(feature = "aura")]
        {
            pallet_session::FindAccountFromAuthorIndex::<Runtime, F>::find_author(_digests)
        }
        #[cfg(feature = "manual-seal")]
        {
            None
        }
    }
}

impl<Runtime, F> FindAuthor<H160> for CoinbaseAuthor<Runtime, F>
where
    Runtime: pallet_session::Config<ValidatorId = AccountId20>,
    F: FindAuthor<u32>,
{
    fn find_author<'a, I>(digests: I) -> Option<H160>
    where
        I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
    {
        <Self as FindAuthor<AccountId>>::find_author(digests).map(Into::into)
    }
}
