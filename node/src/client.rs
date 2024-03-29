use std::sync::Arc;

// Substrate
use sc_client_api::KeyIterator;
use sp_api::NumberFor;
use sp_blockchain::{CachedHeaderMetadata, Info};
use sp_consensus::BlockStatus;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_runtime::{
    generic::{BlockId, SignedBlock},
    traits::{BlakeTwo256, Block as BlockT, Header as HeaderT},
    Justifications,
};
use sp_storage::{ChildInfo, StorageData, StorageKey};
// Local
use primitives_core::{AccountId, Balance, Block, BlockNumber, Hash, Header, Index};

use crate::service::{FullBackend, FullClient};

/// The native executor instance for Eva.
pub struct EvaExecutor;
impl sc_executor::NativeExecutionDispatch for EvaExecutor {
    /// Only enable the benchmarking host functions when we actually want to benchmark.
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    /// Otherwise we only use the default Substrate host functions.
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = primitives_evm_ext::evm_ext::HostFunctions;

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        eva_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        eva_runtime::native_version()
    }
}

/// The native executor instance for Wall-e.
pub struct WallEExecutor;
impl sc_executor::NativeExecutionDispatch for WallEExecutor {
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = primitives_evm_ext::evm_ext::HostFunctions;

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        wall_e_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        wall_e_runtime::native_version()
    }
}

/// A set of APIs that eva-like runtimes must implement.
pub trait RuntimeApiCollection:
    sp_api::ApiExt<Block>
    + sp_api::Metadata<Block>
    + sp_block_builder::BlockBuilder<Block>
    + sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_consensus_aura::AuraApi<Block, AuraId>
    + sp_session::SessionKeys<Block>
    + sp_finality_grandpa::GrandpaApi<Block>
    + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>
    + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
    + primitives_rpc::debug::DebugRuntimeApi<Block>
    + primitives_rpc::txpool::TxPoolRuntimeApi<Block>
    + fp_rpc::EthereumRuntimeRPCApi<Block>
    + fp_rpc::ConvertTransactionRuntimeApi<Block>
where
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api> RuntimeApiCollection for Api
where
    Api: sp_api::ApiExt<Block>
        + sp_api::Metadata<Block>
        + sp_block_builder::BlockBuilder<Block>
        + sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_consensus_aura::AuraApi<Block, AuraId>
        + sp_session::SessionKeys<Block>
        + sp_finality_grandpa::GrandpaApi<Block>
        + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>
        + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
        + primitives_rpc::debug::DebugRuntimeApi<Block>
        + primitives_rpc::txpool::TxPoolRuntimeApi<Block>
        + fp_rpc::EthereumRuntimeRPCApi<Block>
        + fp_rpc::ConvertTransactionRuntimeApi<Block>,
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

/// A client instance of Eva/Wall-e Chain.
///
/// See [`ExecuteWithClient`] for more information.
#[derive(Clone)]
pub enum Client {
    Eva(Arc<FullClient<eva_runtime::RuntimeApi, EvaExecutor>>),
    WallE(Arc<FullClient<wall_e_runtime::RuntimeApi, WallEExecutor>>),
}

/// Unwraps a [`client::Client`] into the concrete runtime client.
#[cfg(feature = "runtime-benchmarks")]
macro_rules! unwrap_client {
    (
        $client:ident,
        $code:expr
    ) => {
        match $client.as_ref() {
            $crate::client::Client::Eva($client) => $code,
            $crate::client::Client::WallE($client) => $code,
        }
    };
}

macro_rules! with_client {
    {
        $self:expr,
        $client:ident,
        $code:expr
    } => {
        match $self {
            Client::Eva($client) => {
                #[allow(unused_imports)]
                use eva_runtime as runtime;
                $code
            },
            Client::WallE($client) => {
                #[allow(unused_imports)]
                use wall_e_runtime as runtime;
                $code
            },
        }
    }
}

type HashFor<B> = <<B as BlockT>::Header as HeaderT>::Hash;

impl sp_blockchain::HeaderBackend<Block> for Client {
    fn header(&self, id: BlockId<Block>) -> sp_blockchain::Result<Option<Header>> {
        with_client! {
            self,
            client,
            {
                client.header(&id)
            }
        }
    }

    fn info(&self) -> Info<Block> {
        with_client! {
            self,
            client,
            {
                client.info()
            }
        }
    }

    fn status(&self, id: BlockId<Block>) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
        with_client! {
            self,
            client,
            {
                client.status(id)
            }
        }
    }

    fn number(&self, hash: Hash) -> sp_blockchain::Result<Option<BlockNumber>> {
        with_client! {
            self,
            client,
            {
                client.number(hash)
            }
        }
    }

    fn hash(&self, number: BlockNumber) -> sp_blockchain::Result<Option<Hash>> {
        with_client! {
            self,
            client,
            {
                client.hash(number)
            }
        }
    }
}

impl sp_blockchain::HeaderMetadata<Block> for Client {
    type Error = sp_blockchain::Error;

    fn header_metadata(
        &self,
        hash: HashFor<Block>,
    ) -> Result<CachedHeaderMetadata<Block>, Self::Error> {
        with_client! {
            self,
            client,
            {
                client.header_metadata(hash)
            }
        }
    }

    fn insert_header_metadata(
        &self,
        hash: HashFor<Block>,
        header_metadata: CachedHeaderMetadata<Block>,
    ) {
        with_client! {
            self,
            client,
            {
                client.insert_header_metadata(hash, header_metadata)
            }
        }
    }

    fn remove_header_metadata(&self, hash: HashFor<Block>) {
        with_client! {
            self,
            client,
            {
                client.remove_header_metadata(hash)
            }
        }
    }
}

impl sc_client_api::AuxStore for Client {
    fn insert_aux<
        'a,
        'b: 'a,
        'c: 'a,
        I: IntoIterator<Item = &'a (&'c [u8], &'c [u8])>,
        D: IntoIterator<Item = &'a &'b [u8]>,
    >(
        &self,
        insert: I,
        delete: D,
    ) -> sp_blockchain::Result<()> {
        with_client! {
            self,
            client,
            {
                client.insert_aux(insert, delete)
            }
        }
    }

    fn get_aux(&self, key: &[u8]) -> sp_blockchain::Result<Option<Vec<u8>>> {
        with_client! {
            self,
            client,
            {
                client.get_aux(key)
            }
        }
    }
}

impl sc_client_api::BlockBackend<Block> for Client {
    fn block_body(
        &self,
        id: &BlockId<Block>,
    ) -> sp_blockchain::Result<Option<Vec<<Block as BlockT>::Extrinsic>>> {
        with_client! {
            self,
            client,
            {
                client.block_body(id)
            }
        }
    }

    fn block(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<SignedBlock<Block>>> {
        with_client! {
            self,
            client,
            {
                client.block(id)
            }
        }
    }

    fn block_status(&self, id: &BlockId<Block>) -> sp_blockchain::Result<BlockStatus> {
        with_client! {
            self,
            client,
            {
                client.block_status(id)
            }
        }
    }

    fn justifications(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<Justifications>> {
        with_client! {
            self,
            client,
            {
                client.justifications(id)
            }
        }
    }

    fn block_hash(
        &self,
        number: NumberFor<Block>,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        with_client! {
            self,
            client,
            {
                client.block_hash(number)
            }
        }
    }

    fn indexed_transaction(
        &self,
        id: &<Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Vec<u8>>> {
        with_client! {
            self,
            client,
            {
                client.indexed_transaction(id)
            }
        }
    }

    fn block_indexed_body(
        &self,
        id: &BlockId<Block>,
    ) -> sp_blockchain::Result<Option<Vec<Vec<u8>>>> {
        with_client! {
            self,
            client,
            {
                client.block_indexed_body(id)
            }
        }
    }

    fn requires_full_sync(&self) -> bool {
        with_client! {
            self,
            client,
            {
                client.requires_full_sync()
            }
        }
    }
}

impl sc_client_api::StorageProvider<Block, FullBackend> for Client {
    fn storage(
        &self,
        id: &BlockId<Block>,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        with_client! {
            self,
            client,
            {
                client.storage(id, key)
            }
        }
    }

    fn storage_keys(
        &self,
        id: &BlockId<Block>,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<StorageKey>> {
        with_client! {
            self,
            client,
            {
                client.storage_keys(id, key_prefix)
            }
        }
    }

    fn storage_hash(
        &self,
        id: &BlockId<Block>,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        with_client! {
            self,
            client,
            {
                client.storage_hash(id, key)
            }
        }
    }

    fn storage_pairs(
        &self,
        id: &BlockId<Block>,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<(StorageKey, StorageData)>> {
        with_client! {
            self,
            client,
            {
                client.storage_pairs(id, key_prefix)
            }
        }
    }

    fn storage_keys_iter<'a>(
        &self,
        id: &BlockId<Block>,
        prefix: Option<&'a StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        KeyIterator<'a, <FullBackend as sc_client_api::Backend<Block>>::State, Block>,
    > {
        with_client! {
            self,
            client,
            {
                client.storage_keys_iter(id, prefix, start_key)
            }
        }
    }

    fn child_storage(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        with_client! {
            self,
            client,
            {
                client.child_storage(id, child_info, key)
            }
        }
    }

    fn child_storage_keys(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<StorageKey>> {
        with_client! {
            self,
            client,
            {
                client.child_storage_keys(id, child_info, key_prefix)
            }
        }
    }

    fn child_storage_keys_iter<'a>(
        &self,
        id: &BlockId<Block>,
        child_info: ChildInfo,
        prefix: Option<&'a StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        KeyIterator<'a, <FullBackend as sc_client_api::Backend<Block>>::State, Block>,
    > {
        with_client! {
            self,
            client,
            {
                client.child_storage_keys_iter(id, child_info, prefix, start_key)
            }
        }
    }

    fn child_storage_hash(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        with_client! {
            self,
            client,
            {
                client.child_storage_hash(id, child_info, key)
            }
        }
    }
}

impl sc_client_api::UsageProvider<Block> for Client {
    fn usage_info(&self) -> sc_client_api::ClientInfo<Block> {
        with_client! {
            self,
            client,
            {
                client.usage_info()
            }
        }
    }
}
