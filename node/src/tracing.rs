use std::{sync::Arc, time::Duration};

use tokio::sync::Semaphore;

// Substrate
use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_service::TaskManager;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, Header as HeaderT};
// Frontier
use fc_rpc::OverrideHandle;
// Local
use amax_eva_rpc::{
    CacheRequester as TraceFilterCacheRequester, CacheTask, DebugHandler, DebugRequester,
    TraceRequester, TraceTask,
};

/// Eth RRC extensions.
#[derive(Clone, PartialEq, Debug)]
pub enum EthApiExt {
    Txpool,
    Debug,
    Trace,
}

impl std::str::FromStr for EthApiExt {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "txpool" => Self::Txpool,
            "debug" => Self::Debug,
            "trace" => Self::Trace,
            _ => return Err(format!("`{}` is not recognized as a supported Ethereum Api", s)),
        })
    }
}

/// RPC configurations
pub struct RpcConfig {
    /// Eth RRC extensions.
    pub ethapi: Vec<EthApiExt>,
    /// Number of concurrent tracing tasks. Meant to be shared by both "debug" and "trace" modules.
    pub ethapi_max_permits: u32,
    /// Duration (in seconds) after which the cache of `trace_filter` for a given block will be
    /// discarded.
    pub ethapi_trace_cache_duration: u64,
}

#[derive(Clone, Default)]
pub struct RpcRequesters {
    pub debug: Option<DebugRequester>,
    pub trace: Option<(TraceRequester, TraceFilterCacheRequester)>,
}

/// Tracing task parameters.
pub struct SpawnTasksParams<'a, B: BlockT, C, BE> {
    pub task_manager: &'a TaskManager,
    pub client: Arc<C>,
    pub substrate_backend: Arc<BE>,
    pub frontier_backend: Arc<fc_db::Backend<B>>,
    pub overrides: Arc<OverrideHandle<B>>,
}

// Spawn the tasks that are required to run a tracing node.
pub fn spawn_tracing_tasks<B, C, BE>(
    config: &RpcConfig,
    params: SpawnTasksParams<B, C, BE>,
) -> RpcRequesters
where
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    B::Header: HeaderT<Number = u32>,
    C: ProvideRuntimeApi<B> + StorageProvider<B, BE>,
    C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: BlockBuilder<B>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<B>,
    C::Api: primitives_rpc::debug::DebugRuntimeApi<B>,
    BE: Backend<B> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
{
    let permit_pool = Arc::new(Semaphore::new(config.ethapi_max_permits as usize));

    let (trace_task, trace_filter_task, trace_requesters) = if config
        .ethapi
        .contains(&EthApiExt::Trace)
    {
        let (trace_task, trace_requester) = TraceTask::create(
            params.client.clone(),
            params.substrate_backend.clone(),
            params.frontier_backend.clone(),
            permit_pool.clone(),
            params.overrides.clone(),
        );

        let (trace_filter_task, trace_filter_requester) = CacheTask::create(
            params.client.clone(),
            params.substrate_backend.clone(),
            Duration::from_secs(config.ethapi_trace_cache_duration),
            permit_pool.clone(),
            params.overrides.clone(),
        );

        (Some(trace_task), Some(trace_filter_task), Some((trace_requester, trace_filter_requester)))
    } else {
        (None, None, None)
    };

    let (debug_task, debug_requester) = if config.ethapi.contains(&EthApiExt::Debug) {
        let (debug_task, debug_requester) = DebugHandler::task(
            params.client.clone(),
            params.substrate_backend.clone(),
            params.frontier_backend.clone(),
            permit_pool.clone(),
            params.overrides.clone(),
        );
        (Some(debug_task), Some(debug_requester))
    } else {
        (None, None)
    };

    // `trace_filter` cache task. Essential.
    // Proxies rpc requests to it's handler.
    if let Some(trace_filter_task) = trace_filter_task {
        params.task_manager.spawn_essential_handle().spawn(
            "ethapi-trace-filter-cache",
            Some("eth-tracing"),
            trace_filter_task,
        );
    }

    // trace task. Essential.
    // Proxies rpc requests to it's handler.
    if let Some(trace_task) = trace_task {
        params.task_manager.spawn_essential_handle().spawn(
            "ethapi-trace-task",
            Some("eth-tracing"),
            trace_task,
        );
    }

    // `debug` task if enabled. Essential.
    // Proxies rpc requests to it's handler.
    if let Some(debug_task) = debug_task {
        params.task_manager.spawn_essential_handle().spawn(
            "ethapi-debug",
            Some("eth-tracing"),
            debug_task,
        );
    }

    RpcRequesters { debug: debug_requester, trace: trace_requesters }
}

pub fn rpc_requesters<B, C, BE>(
    ethapi: &[EthApiExt],
    config: &RpcConfig,
    params: SpawnTasksParams<B, C, BE>,
) -> RpcRequesters
where
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    B::Header: HeaderT<Number = u32>,
    C: ProvideRuntimeApi<B> + StorageProvider<B, BE>,
    C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError> + 'static,
    C::Api: sp_block_builder::BlockBuilder<B>,
    C::Api: fp_rpc::EthereumRuntimeRPCApi<B>,
    C::Api: primitives_rpc::debug::DebugRuntimeApi<B>,
    BE: Backend<B> + 'static,
    BE::State: StateBackend<BlakeTwo256>,
{
    if ethapi.contains(&EthApiExt::Debug) || ethapi.contains(&EthApiExt::Trace) {
        spawn_tracing_tasks(config, params)
    } else {
        RpcRequesters::default()
    }
}
