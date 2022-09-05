//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::{future, StreamExt};
// Substrate
use sc_cli::SubstrateCli;
use sc_client_api::{BlockchainEvents, StateBackendFor};
use sc_executor::{NativeElseWasmExecutor, NativeExecutionDispatch};
use sc_service::{
    error::Error as ServiceError, BasePath, Configuration, PartialComponents, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
// Frontier
use fc_consensus::FrontierBlockImport;
use fc_db::Backend as FrontierBackend;
use fc_mapping_sync::{MappingSyncWorker, SyncStrategy};
use fc_rpc::{EthTask, OverrideHandle};
use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};
use sp_api::ConstructRuntimeApi;
use sp_runtime::traits::BlakeTwo256;
use sp_trie::PrefixedMemoryDB;
// Local
use primitives_core::Block;

#[cfg(feature = "manual-seal")]
use crate::cli::Sealing;
use crate::{
    chain_spec::{RuntimeChain, RuntimeChainSpec},
    cli::Cli,
    client::{Client, EvaExecutor, RuntimeApiCollection, WallEExecutor},
};

pub type FullClient<RuntimeApi, Executor> =
    sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;
pub type FullBackend = sc_service::TFullBackend<Block>;
pub type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

#[cfg(feature = "aura")]
pub type ConsensusResult<RuntimeApi, Executor> = (
    FrontierBlockImport<
        Block,
        sc_finality_grandpa::GrandpaBlockImport<
            FullBackend,
            Block,
            FullClient<RuntimeApi, Executor>,
            FullSelectChain,
        >,
        FullClient<RuntimeApi, Executor>,
    >,
    sc_finality_grandpa::LinkHalf<Block, FullClient<RuntimeApi, Executor>, FullSelectChain>,
);

#[cfg(feature = "manual-seal")]
pub type ConsensusResult<RuntimeApi, Executor> = (
    FrontierBlockImport<
        Block,
        Arc<FullClient<RuntimeApi, Executor>>,
        FullClient<RuntimeApi, Executor>,
    >,
    Sealing,
);

pub(crate) fn db_config_dir(config: &Configuration) -> PathBuf {
    config
        .base_path
        .as_ref()
        .map(|base_path| base_path.config_dir(config.chain_spec.id()))
        .unwrap_or_else(|| {
            BasePath::from_project("", "", &Cli::executable_name())
                .config_dir(config.chain_spec.id())
        })
}

pub fn new_partial<RuntimeApi, Executor>(
    config: &Configuration,
    cli: &Cli,
) -> Result<
    PartialComponents<
        FullClient<RuntimeApi, Executor>,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
        sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
        (
            Option<Telemetry>,
            ConsensusResult<RuntimeApi, Executor>,
            Arc<fc_db::Backend<Block>>,
            Option<FilterPool>,
            (FeeHistoryCache, FeeHistoryCacheLimit),
        ),
    >,
    ServiceError,
>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
{
    if config.keystore_remote.is_some() {
        return Err(ServiceError::Other("Remote Keystores are not supported.".into()))
    }

    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = NativeElseWasmExecutor::<Executor>::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
        config.runtime_cache_size,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    // Frontier
    let frontier_backend =
        Arc::new(FrontierBackend::open(&config.database, &db_config_dir(config))?);
    let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));
    let fee_history_cache: FeeHistoryCache = Arc::new(Mutex::new(BTreeMap::new()));
    let fee_history_cache_limit: FeeHistoryCacheLimit = cli.run.fee_history_limit;

    #[cfg(feature = "aura")]
    {
        use sc_client_api::ExecutorProvider;
        use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;

        let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
            client.clone(),
            &(client.clone() as Arc<_>),
            select_chain.clone(),
            telemetry.as_ref().map(|x| x.handle()),
        )?;

        let frontier_block_import = FrontierBlockImport::new(
            grandpa_block_import.clone(),
            client.clone(),
            frontier_backend.clone(),
        );

        let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
        let create_inherent_data_providers = move |_, ()| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            let slot =
            sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                slot_duration,
            );
            Ok((timestamp, slot))
        };
        let import_queue = sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _, _>(
            sc_consensus_aura::ImportQueueParams {
                block_import: frontier_block_import.clone(),
                justification_import: Some(Box::new(grandpa_block_import)),
                client: client.clone(),
                create_inherent_data_providers,
                spawner: &task_manager.spawn_essential_handle(),
                can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(
                    client.executor().clone(),
                ),
                registry: config.prometheus_registry(),
                check_for_equivocation: Default::default(),
                telemetry: telemetry.as_ref().map(|x| x.handle()),
            },
        )?;

        Ok(sc_service::PartialComponents {
            client,
            backend,
            task_manager,
            import_queue,
            keystore_container,
            select_chain,
            transaction_pool,
            other: (
                telemetry,
                (frontier_block_import, grandpa_link),
                frontier_backend,
                filter_pool,
                (fee_history_cache, fee_history_cache_limit),
            ),
        })
    }

    #[cfg(feature = "manual-seal")]
    {
        let sealing = cli.run.sealing;

        let frontier_block_import =
            FrontierBlockImport::new(client.clone(), client.clone(), frontier_backend.clone());

        let import_queue = sc_consensus_manual_seal::import_queue(
            Box::new(frontier_block_import.clone()),
            &task_manager.spawn_essential_handle(),
            config.prometheus_registry(),
        );

        Ok(sc_service::PartialComponents {
            client,
            backend,
            task_manager,
            import_queue,
            keystore_container,
            select_chain,
            transaction_pool,
            other: (
                telemetry,
                (frontier_block_import, sealing),
                frontier_backend,
                filter_pool,
                (fee_history_cache, fee_history_cache_limit),
            ),
        })
    }
}

/// Builds a new service for a full client.
#[cfg(feature = "aura")]
pub fn new_full<RuntimeApi, Executor>(
    mut config: Configuration,
    cli: &Cli,
) -> Result<TaskManager, ServiceError>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
    Executor: NativeExecutionDispatch + 'static,
{
    use sc_client_api::{BlockBackend, ExecutorProvider};
    use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;

    let PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other:
            (
                mut telemetry,
                (block_import, grandpa_link),
                frontier_backend,
                filter_pool,
                (fee_history_cache, fee_history_cache_limit),
            ),
    } = new_partial::<RuntimeApi, Executor>(&config, cli)?;

    let grandpa_protocol_name = sc_finality_grandpa::protocol_standard_name(
        &client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"),
        &config.chain_spec,
    );
    config
        .network
        .extra_sets
        .push(sc_finality_grandpa::grandpa_peers_set_config(grandpa_protocol_name.clone()));

    let warp_sync = Arc::new(sc_finality_grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        grandpa_link.shared_authority_set().clone(),
        Vec::default(),
    ));

    let (network, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync: Some(warp_sync),
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let role = config.role.clone();
    let force_authoring = config.force_authoring;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();
    // Frontier
    let overrides = crate::rpc::overrides_handle(client.clone());
    let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
        task_manager.spawn_handle(),
        overrides.clone(),
        50,
        50,
        prometheus_registry.clone(),
    ));

    let rpc_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();
        let is_authority = role.is_authority();
        let enable_dev_signer = cli.run.enable_dev_signer;
        let network = network.clone();
        let filter_pool = filter_pool.clone();
        let frontier_backend = frontier_backend.clone();
        let overrides = overrides.clone();
        let fee_history_cache = fee_history_cache.clone();
        let max_past_logs = cli.run.max_past_logs;
        let ethapi = cli.run.ethapi.clone();
        let tracing_requesters = crate::tracing::rpc_requesters(
            &ethapi,
            &crate::tracing::RpcConfig {
                ethapi: ethapi.clone(),
                ethapi_max_permits: cli.run.ethapi_max_permits,
                ethapi_trace_cache_duration: cli.run.ethapi_trace_cache_duration,
            },
            crate::tracing::SpawnTasksParams {
                task_manager: &task_manager,
                client: client.clone(),
                substrate_backend: backend.clone(),
                frontier_backend: frontier_backend.clone(),
                overrides: overrides.clone(),
            },
        );
        let trace_filter_max_count = cli.run.ethapi_trace_max_count;
        let chain = config.chain_spec.runtime();

        Box::new(move |deny_unsafe, subscription_task_executor| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                deny_unsafe,
                graph: pool.pool().clone(),
                is_authority,
                enable_dev_signer,
                network: network.clone(),
                filter_pool: filter_pool.clone(),
                backend: frontier_backend.clone(),
                max_past_logs,
                tracing_requesters: tracing_requesters.clone(),
                trace_filter_max_count,
                fee_history_cache: fee_history_cache.clone(),
                fee_history_cache_limit,
                execute_gas_limit_multiplier: 10,
                overrides: overrides.clone(),
                block_data_cache: block_data_cache.clone(),
                chain,
            };

            crate::rpc::create_full(deps, subscription_task_executor, ethapi.clone())
                .map_err(Into::into)
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: network.clone(),
        client: client.clone(),
        keystore: keystore_container.sync_keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder,
        backend: backend.clone(),
        system_rpc_tx,
        config,
        telemetry: telemetry.as_mut(),
    })?;

    spawn_frontier_tasks(
        &task_manager,
        client.clone(),
        backend,
        frontier_backend,
        filter_pool,
        overrides,
        fee_history_cache,
        fee_history_cache_limit,
    );

    if role.is_authority() {
        let proposer_factory = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
        let create_inherent_data_providers = move |_, ()| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                slot_duration,
            );
            Ok((timestamp, slot))
        };
        let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _, _, _>(
            sc_consensus_aura::StartAuraParams {
                slot_duration,
                client: client.clone(),
                select_chain,
                block_import,
                proposer_factory,
                sync_oracle: network.clone(),
                justification_sync_link: network.clone(),
                create_inherent_data_providers,
                force_authoring,
                backoff_authoring_blocks: Option::<()>::None,
                keystore: keystore_container.sync_keystore(),
                can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(
                    client.executor().clone(),
                ),
                block_proposal_slot_portion: sc_consensus_aura::SlotProportion::new(2f32 / 3f32),
                max_block_proposal_slot_portion: None,
                telemetry: telemetry.as_ref().map(|x| x.handle()),
            },
        )?;
        // the AURA authoring task is considered essential, i.e. if it
        // fails we take down the service with it.
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("aura", Some("block-authoring"), aura);
    }

    if enable_grandpa {
        // if the node isn't actively participating in consensus then it doesn't
        // need a keystore, regardless of which protocol we use below.
        let keystore =
            if role.is_authority() { Some(keystore_container.sync_keystore()) } else { None };

        let grandpa_config = sc_finality_grandpa::Config {
            // FIXME #1578 make this available through chainspec
            gossip_duration: Duration::from_millis(333),
            justification_period: 512,
            name: Some(name),
            observer_enabled: false,
            keystore,
            local_role: role,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            protocol_name: grandpa_protocol_name,
        };

        // start the full GRANDPA voter
        // NOTE: non-authorities could run the GRANDPA observer protocol, but at
        // this point the full voter should provide better guarantees of block
        // and vote data availability than the observer. The observer has not
        // been tested extensively yet and having most nodes in a network run it
        // could lead to finality stalls.
        let grandpa_voter =
            sc_finality_grandpa::run_grandpa_voter(sc_finality_grandpa::GrandpaParams {
                config: grandpa_config,
                link: grandpa_link,
                network,
                voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
                prometheus_registry,
                shared_voter_state: sc_finality_grandpa::SharedVoterState::empty(),
                telemetry: telemetry.as_ref().map(|x| x.handle()),
            })?;

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("grandpa-voter", None, grandpa_voter);
    }

    network_starter.start_network();
    Ok(task_manager)
}

/// Builds a new service for a full client.
#[cfg(feature = "manual-seal")]
pub fn new_full<RuntimeApi, Executor>(
    config: Configuration,
    cli: &Cli,
    chain: Chain,
) -> Result<TaskManager, ServiceError>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,

    Executor: NativeExecutionDispatch + 'static,
{
    let PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        mut keystore_container,
        select_chain,
        transaction_pool,
        other:
            (
                mut telemetry,
                (block_import, sealing),
                frontier_backend,
                filter_pool,
                (fee_history_cache, fee_history_cache_limit),
            ),
    } = new_partial::<RuntimeApi, Executor>(&config, cli)?;

    if let Some(url) = &config.keystore_remote {
        match remote_keystore(url) {
            Ok(k) => keystore_container.set_remote_keystore(k),
            Err(e) => {
                return Err(ServiceError::Other(format!(
                    "Error hooking up remote keystore for {}: {}",
                    url, e
                )))
            },
        };
    }

    let (network, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync: None,
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let role = config.role.clone();
    let prometheus_registry = config.prometheus_registry().cloned();
    // Frontier
    let overrides = crate::rpc::overrides_handle(client.clone());
    let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
        task_manager.spawn_handle(),
        overrides.clone(),
        50,
        50,
        prometheus_registry.clone(),
    ));

    // Channel for the rpc handler to communicate with the authorship task.
    let (command_sink, commands_stream) = futures::channel::mpsc::channel(1000);

    let rpc_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();
        let is_authority = role.is_authority();
        let enable_dev_signer = cli.run.enable_dev_signer;
        let network = network.clone();
        let filter_pool = filter_pool.clone();
        let frontier_backend = frontier_backend.clone();
        let overrides = overrides.clone();
        let fee_history_cache = fee_history_cache.clone();
        let max_past_logs = cli.run.max_past_logs;
        let ethapi = cli.run.ethapi.clone();
        let tracing_requesters = crate::tracing::rpc_requesters(
            &ethapi,
            &crate::tracing::RpcConfig {
                ethapi: ethapi.clone(),
                ethapi_max_permits: cli.run.ethapi_max_permits,
                ethapi_trace_cache_duration: cli.run.ethapi_trace_cache_duration,
            },
            crate::tracing::SpawnTasksParams {
                task_manager: &task_manager,
                client: client.clone(),
                substrate_backend: backend.clone(),
                frontier_backend: frontier_backend.clone(),
                overrides: overrides.clone(),
            },
        );
        let trace_filter_max_count = cli.run.ethapi_trace_max_count;
        let chain = config.chain_spec.runtime();

        Box::new(move |deny_unsafe, subscription_task_executor| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                graph: pool.pool().clone(),
                deny_unsafe,
                is_authority,
                enable_dev_signer,
                network: network.clone(),
                filter_pool: filter_pool.clone(),
                backend: frontier_backend.clone(),
                max_past_logs,
                tracing_requesters: tracing_requesters.clone(),
                trace_filter_max_count,
                fee_history_cache: fee_history_cache.clone(),
                fee_history_cache_limit,
                execute_gas_limit_multiplier: 10,
                overrides: overrides.clone(),
                block_data_cache: block_data_cache.clone(),
                chain,
                command_sink: Some(command_sink.clone()),
            };

            crate::rpc::create_full(deps, subscription_task_executor, ethapi.clone())
                .map_err(Into::into)
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network,
        client: client.clone(),
        keystore: keystore_container.sync_keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder,
        backend: backend.clone(),
        system_rpc_tx,
        config,
        telemetry: telemetry.as_mut(),
    })?;

    spawn_frontier_tasks(
        &task_manager,
        client.clone(),
        backend,
        frontier_backend,
        filter_pool,
        overrides,
        fee_history_cache,
        fee_history_cache_limit,
    );

    if role.is_authority() {
        let env = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let create_inherent_data_providers = move |_, ()| async move {
            let mock_timestamp = crate::manual_seal::MockTimestampInherentDataProvider;
            Ok(mock_timestamp)
        };

        let manual_seal = match sealing {
            Sealing::Manual => future::Either::Left(sc_consensus_manual_seal::run_manual_seal(
                sc_consensus_manual_seal::ManualSealParams {
                    block_import,
                    env,
                    client,
                    pool: transaction_pool,
                    commands_stream,
                    select_chain,
                    consensus_data_provider: None,
                    create_inherent_data_providers,
                },
            )),
            Sealing::Instant => future::Either::Right(sc_consensus_manual_seal::run_instant_seal(
                sc_consensus_manual_seal::InstantSealParams {
                    block_import,
                    env,
                    client,
                    pool: transaction_pool,
                    select_chain,
                    consensus_data_provider: None,
                    create_inherent_data_providers,
                },
            )),
        };
        // we spawn the future on a background thread managed by service.
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("manual-seal", None, manual_seal);
    }

    log::info!("Manual Seal Ready");

    network_starter.start_network();
    Ok(task_manager)
}

pub fn build_full(config: Configuration, cli: &Cli) -> Result<TaskManager, ServiceError> {
    match config.chain_spec.runtime() {
        RuntimeChainSpec::Eva => new_full::<eva_runtime::RuntimeApi, EvaExecutor>(config, cli),
        RuntimeChainSpec::WallE => {
            new_full::<wall_e_runtime::RuntimeApi, WallEExecutor>(config, cli)
        },
        RuntimeChainSpec::Unknown => panic!("Unknown chain spec"),
    }
}

pub fn new_chain_ops(
    config: &mut Configuration,
    cli: &Cli,
) -> Result<
    (
        Arc<Client>,
        Arc<FullBackend>,
        sc_consensus::import_queue::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
        TaskManager,
    ),
    ServiceError,
> {
    config.keystore = sc_service::config::KeystoreConfig::InMemory;
    match config.chain_spec.runtime() {
        RuntimeChainSpec::Eva => {
            let PartialComponents { client, backend, import_queue, task_manager, .. } =
                new_partial::<eva_runtime::RuntimeApi, EvaExecutor>(config, cli)?;
            Ok((Arc::new(Client::Eva(client)), backend, import_queue, task_manager))
        },
        RuntimeChainSpec::WallE => {
            let PartialComponents { client, backend, import_queue, task_manager, .. } =
                new_partial::<wall_e_runtime::RuntimeApi, WallEExecutor>(config, cli)?;
            Ok((Arc::new(Client::WallE(client)), backend, import_queue, task_manager))
        },
        RuntimeChainSpec::Unknown => panic!("Unknown chain spec"),
    }
}

// Spawn frontier tasks.
pub fn spawn_frontier_tasks<RuntimeApi, Executor>(
    task_manager: &TaskManager,
    client: Arc<FullClient<RuntimeApi, Executor>>,
    backend: Arc<FullBackend>,
    frontier_backend: Arc<fc_db::Backend<Block>>,
    filter_pool: Option<FilterPool>,
    overrides: Arc<OverrideHandle<Block>>,
    fee_history_cache: FeeHistoryCache,
    fee_history_cache_limit: u64,
) where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: NativeExecutionDispatch + 'static,
{
    task_manager.spawn_essential_handle().spawn(
        "frontier-mapping-sync-worker",
        Some("frontier"),
        MappingSyncWorker::new(
            client.import_notification_stream(),
            Duration::new(2, 0),
            client.clone(),
            backend,
            frontier_backend,
            3,
            0,
            SyncStrategy::Normal,
        )
        .for_each(|()| future::ready(())),
    );

    // Spawn Frontier EthFilterApi maintenance task.
    if let Some(filter_pool) = filter_pool {
        // Each filter is allowed to stay in the pool for 100 blocks.
        const FILTER_RETAIN_THRESHOLD: u64 = 100;
        task_manager.spawn_essential_handle().spawn(
            "frontier-filter-pool",
            Some("frontier"),
            EthTask::filter_pool_task(client.clone(), filter_pool, FILTER_RETAIN_THRESHOLD),
        );
    }

    // Spawn Frontier FeeHistory cache maintenance task.
    task_manager.spawn_essential_handle().spawn(
        "frontier-fee-history",
        Some("frontier"),
        EthTask::fee_history_task(client, overrides, fee_history_cache, fee_history_cache_limit),
    );
}
