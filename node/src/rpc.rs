//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

use std::{collections::BTreeMap, sync::Arc};

use jsonrpsee::RpcModule;
// Substrate
use sc_client_api::{client::BlockchainEvents, AuxStore, Backend, StateBackend, StorageProvider};
use sc_network::NetworkService;
use sc_rpc::SubscriptionTaskExecutor;
use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::traits::BlakeTwo256;
// Frontier
use fc_rpc::{
    EthBlockDataCacheTask, OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override,
    SchemaV2Override, SchemaV3Override, StorageOverride,
};
use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};
use fp_storage::EthereumStorageSchema;
// Local
use primitives_core::{AccountId, Balance, Block, Chain, Hash, Index};
use runtime_common::EthereumTransaction;

use amax_eva_rpc::{Debug as DebugRpc, DebugApiServer, TxPool as TxPoolRpc, TxPoolApiServer};

use crate::tracing::RpcRequesters as TracingRpcRequesters;
pub use crate::tracing::{EthApiExt, RpcConfig};

enum TransactionConverter {
    Eva(eva_runtime::TransactionConverter),
    WallE(wall_e_runtime::TransactionConverter),
}
impl fp_rpc::ConvertTransaction<primitives_core::UncheckedExtrinsic> for TransactionConverter {
    fn convert_transaction(
        &self,
        transaction: EthereumTransaction,
    ) -> primitives_core::UncheckedExtrinsic {
        match &self {
            Self::Eva(inner) => inner.convert_transaction(transaction),
            Self::WallE(inner) => inner.convert_transaction(transaction),
        }
    }
}

impl From<Chain> for TransactionConverter {
    fn from(chain: Chain) -> Self {
        match chain {
            Chain::Eva => Self::Eva(eva_runtime::TransactionConverter::new()),
            Chain::WallE => Self::WallE(wall_e_runtime::TransactionConverter::new()),
        }
    }
}

/// Full client dependencies.
pub struct FullDeps<C, P, A: ChainApi> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// Graph pool instance.
    pub graph: Arc<Pool<A>>,
    /// The Node authority flag
    pub is_authority: bool,
    /// Whether to enable dev signer
    pub enable_dev_signer: bool,
    /// Network service
    pub network: Arc<NetworkService<Block, Hash>>,
    /// EthFilterApi pool.
    pub filter_pool: Option<FilterPool>,
    /// Backend.
    pub backend: Arc<fc_db::Backend<Block>>,
    /// Maximum number of logs in a query.
    pub max_past_logs: u32,
    /// tracing requesters
    pub tracing_requesters: TracingRpcRequesters,
    /// trace filter max count
    pub trace_filter_max_count: u32,
    /// Fee history cache.
    pub fee_history_cache: FeeHistoryCache,
    /// Maximum fee history cache size.
    pub fee_history_cache_limit: FeeHistoryCacheLimit,
    /// Ethereum data access overrides.
    pub overrides: Arc<OverrideHandle<Block>>,
    /// Cache for Ethereum block data.
    pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
    /// Amax Chain Type
    pub chain: Chain,
    /// Manual seal command sink
    #[cfg(feature = "manual-seal")]
    pub command_sink:
        Option<futures::channel::mpsc::Sender<sc_consensus_manual_seal::rpc::EngineCommand<Hash>>>,
}

pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<OverrideHandle<Block>>
where
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
{
    let mut overrides_map = BTreeMap::new();
    overrides_map.insert(
        EthereumStorageSchema::V1,
        Box::new(SchemaV1Override::new(client.clone()))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );
    overrides_map.insert(
        EthereumStorageSchema::V2,
        Box::new(SchemaV2Override::new(client.clone()))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );
    overrides_map.insert(
        EthereumStorageSchema::V3,
        Box::new(SchemaV3Override::new(client.clone()))
            as Box<dyn StorageOverride<_> + Send + Sync>,
    );

    Arc::new(OverrideHandle {
        schemas: overrides_map,
        fallback: Box::new(RuntimeApiStorageOverride::new(client)),
    })
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P, BE, A>(
    deps: FullDeps<C, P, A>,
    subscription_task_executor: SubscriptionTaskExecutor,
    exts: Vec<EthApiExt>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    BE: Backend<Block> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
    C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
    C: BlockchainEvents<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: BlockBuilder<Block>,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
    C::Api: primitives_rpc::txpool::TxPoolRuntimeApi<Block>,
    P: TransactionPool<Block = Block> + 'static,
    A: ChainApi<Block = Block> + 'static,
{
    // Substrate
    use pallet_transaction_payment_rpc::{TransactionPaymentApiServer, TransactionPaymentRpc};
    #[cfg(feature = "manual-seal")]
    use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApiServer};
    use substrate_frame_rpc_system::{SystemApiServer, SystemRpc};
    // Frontier
    use fc_rpc::{
        Eth, EthApiServer, EthDevSigner, EthFilter, EthFilterApiServer, EthPubSub,
        EthPubSubApiServer, EthSigner, Net, NetApiServer, Web3, Web3ApiServer,
    };
    // Local
    use amax_eva_rpc::{Trace, TraceServer};

    let mut io = RpcModule::new(());
    let FullDeps {
        client,
        pool,
        deny_unsafe,
        graph,
        is_authority,
        enable_dev_signer,
        network,
        filter_pool,
        backend,
        max_past_logs,
        tracing_requesters,
        trace_filter_max_count,
        fee_history_cache,
        fee_history_cache_limit,
        overrides,
        block_data_cache,
        chain,
        #[cfg(feature = "manual-seal")]
        command_sink,
    } = deps;

    io.merge(SystemRpc::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
    io.merge(TransactionPaymentRpc::new(client.clone()).into_rpc())?;

    let mut signers = Vec::new();
    if enable_dev_signer {
        signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
    }

    io.merge(
        Eth::new(
            client.clone(),
            pool.clone(),
            graph.clone(),
            Some(TransactionConverter::from(chain)),
            network.clone(),
            signers,
            overrides.clone(),
            backend.clone(),
            is_authority,
            block_data_cache.clone(),
            fee_history_cache,
            fee_history_cache_limit,
            fc_rpc::format::Geth,
        )
        .into_rpc(),
    )?;
    if let Some(filter_pool) = filter_pool {
        io.merge(
            EthFilter::new(
                client.clone(),
                backend,
                filter_pool,
                500, // max stored filters
                max_past_logs,
                block_data_cache,
            )
            .into_rpc(),
        )?;
    }
    io.merge(
        EthPubSub::new(
            pool,
            client.clone(),
            network.clone(),
            subscription_task_executor,
            overrides,
        )
        .into_rpc(),
    )?;
    io.merge(Net::new(client.clone(), network, true).into_rpc())?;
    io.merge(Web3::new(client.clone()).into_rpc())?;

    if let Some((trace_requester, trace_filter_requester)) = tracing_requesters.trace {
        io.merge(
            Trace::new(
                client.clone(),
                trace_filter_requester,
                trace_requester,
                trace_filter_max_count,
            )
            .into_rpc(),
        )?;
    }

    if exts.contains(&EthApiExt::Txpool) {
        io.merge(TxPoolRpc::new(client, graph).into_rpc())?;
    }

    if let Some(debug_requester) = tracing_requesters.debug {
        io.merge(DebugRpc::new(debug_requester).into_rpc())?;
    }

    #[cfg(feature = "manual-seal")]
    if let Some(command_sink) = command_sink {
        io.merge(
            // We provide the rpc handler with the sending end of the channel to allow the rpc
            // send EngineCommands to the background block authorship task.
            ManualSeal::new(command_sink).into_rpc(),
        )?;
    }

    Ok(io)
}
