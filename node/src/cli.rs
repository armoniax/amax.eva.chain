use std::net::SocketAddr;

use sc_cli::{
    build_runtime, ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams,
    KeystoreParams, NetworkParams, OffchainWorkerParams, Result, Role, Runner, SharedParams,
    SubstrateCli,
};
use sc_service::{config::PrometheusConfig, BasePath, TransactionPoolOptions};
use sc_telemetry::TelemetryEndpoints;

/// Armonia Eva Node CLI.
#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[doc(hidden)]
    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[doc(hidden)]
    #[clap(flatten)]
    pub run: RunCmd,
}

/// Available Sealing methods.
#[cfg(feature = "manual-seal")]
#[derive(Debug, Copy, Clone, clap::ArgEnum)]
pub enum Sealing {
    /// Seal using rpc method.
    Manual,
    /// Seal when transaction is executed.
    Instant,
}

#[cfg(feature = "manual-seal")]
impl Default for Sealing {
    fn default() -> Sealing {
        Sealing::Instant
    }
}

/// The `run` command used to run a node.
#[derive(Debug, clap::Parser)]
pub struct RunCmd {
    #[doc(hidden)]
    #[clap(flatten)]
    pub base: sc_cli::RunCmd,

    /// Choose sealing method.
    #[cfg(feature = "manual-seal")]
    #[clap(long, arg_enum, ignore_case = true, default_value_t)]
    pub sealing: Sealing,

    /// Enable dev signer for eth rpc.
    #[clap(long)]
    pub enable_dev_signer: bool,

    /// Maximum number of logs in a query.
    #[clap(long, default_value = "10000")]
    pub max_past_logs: u32,

    /// Maximum fee history cache size.
    #[clap(long, default_value = "2048")]
    pub fee_history_limit: u64,
}

/// Armonia Eva Node subcommand.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Key management cli utilities
    #[clap(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Sub-commands concerned with benchmarking.
    #[cfg(feature = "runtime-benchmarks")]
    #[clap(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Sub-commands concerned with benchmarking.
    /// Note: `runtime-benchmarks` feature must be enabled.
    #[cfg(not(feature = "runtime-benchmarks"))]
    Benchmark,

    /// Try some command against runtime state.
    #[cfg(feature = "try-runtime")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),

    /// Try some command against runtime state.
    /// Note: `try-runtime` feature must be enabled.
    #[cfg(not(feature = "try-runtime"))]
    TryRuntime,
}

#[derive(Copy, Clone)]
enum ChainNetworkType {
    Dev,
    Testnet,
    Mainnet,
}

static mut CHAIN_NETWORK_TYPE: ChainNetworkType = ChainNetworkType::Mainnet;
fn set_chain_network_type(network_type: ChainNetworkType) {
    // this is safe, for this function should only be called in `create_runner_for_run_cmd`.
    unsafe {
        CHAIN_NETWORK_TYPE = network_type;
    }
}
fn get_chain_network_type() -> ChainNetworkType {
    // this is safe, for this function is not written when called.
    unsafe { CHAIN_NETWORK_TYPE }
}

impl DefaultConfigurationValues for Cli {
    fn p2p_listen_port() -> u16 {
        match get_chain_network_type() {
            ChainNetworkType::Dev => 30333,
            ChainNetworkType::Mainnet => 9922,
            ChainNetworkType::Testnet => 19922,
        }
    }

    fn rpc_ws_listen_port() -> u16 {
        match get_chain_network_type() {
            ChainNetworkType::Dev | ChainNetworkType::Mainnet => 9944,
            ChainNetworkType::Testnet => 19944,
        }
    }

    fn rpc_http_listen_port() -> u16 {
        match get_chain_network_type() {
            ChainNetworkType::Dev | ChainNetworkType::Mainnet => 9933,
            ChainNetworkType::Testnet => 19933,
        }
    }

    fn prometheus_listen_port() -> u16 {
        9616
    }
}

/// A copy implementation for RunCmd, but the generic type `DCV` in `CliConfiguration<DCV>` is
/// `Cli`. Notice this implementation should be updated if the `CliConfiguration` implementation for
/// `RunCmd` has changed.
/// More exactly, the function which should be implemented for `Cli` is the one that be called in
/// `create_configuration` function, while it's also overridden by `RunCmd`.
/// Thus, the easiest way is to override all functions which are overridden for `RunCmd`.
impl CliConfiguration<Self> for Cli {
    fn shared_params(&self) -> &SharedParams {
        self.run.base.shared_params()
    }
    fn import_params(&self) -> Option<&ImportParams> {
        self.run.base.import_params()
    }
    fn network_params(&self) -> Option<&NetworkParams> {
        self.run.base.network_params()
    }
    fn keystore_params(&self) -> Option<&KeystoreParams> {
        self.run.base.keystore_params()
    }
    fn offchain_worker_params(&self) -> Option<&OffchainWorkerParams> {
        self.run.base.offchain_worker_params()
    }
    fn node_name(&self) -> Result<String> {
        self.run.base.node_name()
    }
    fn dev_key_seed(&self, is_dev: bool) -> Result<Option<String>> {
        self.run.base.dev_key_seed(is_dev)
    }
    fn telemetry_endpoints(
        &self,
        chain_spec: &Box<dyn ChainSpec>,
    ) -> Result<Option<TelemetryEndpoints>> {
        self.run.base.telemetry_endpoints(chain_spec)
    }
    fn role(&self, is_dev: bool) -> Result<Role> {
        self.run.base.role(is_dev)
    }
    fn force_authoring(&self) -> Result<bool> {
        self.run.base.force_authoring()
    }
    fn prometheus_config(
        &self,
        default_listen_port: u16,
        chain_spec: &Box<dyn ChainSpec>,
    ) -> Result<Option<PrometheusConfig>> {
        self.run.base.prometheus_config(default_listen_port, chain_spec)
    }
    fn disable_grandpa(&self) -> Result<bool> {
        self.run.base.disable_grandpa()
    }
    fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
        self.run.base.rpc_ws_max_connections()
    }

    fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
        self.run.base.rpc_cors(is_dev)
    }
    fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
        self.run.base.rpc_http(default_listen_port)
    }
    fn rpc_ipc(&self) -> Result<Option<String>> {
        self.run.base.rpc_ipc()
    }
    fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
        self.run.base.rpc_ws(default_listen_port)
    }
    fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
        self.run.base.rpc_methods()
    }
    fn rpc_max_payload(&self) -> Result<Option<usize>> {
        self.run.base.rpc_max_payload()
    }

    fn ws_max_out_buffer_capacity(&self) -> Result<Option<usize>> {
        self.run.base.ws_max_out_buffer_capacity()
    }
    fn transaction_pool(&self) -> Result<TransactionPoolOptions> {
        self.run.base.transaction_pool()
    }
    fn max_runtime_instances(&self) -> Result<Option<usize>> {
        self.run.base.max_runtime_instances()
    }
    fn runtime_cache_size(&self) -> Result<u8> {
        self.run.base.runtime_cache_size()
    }
    fn base_path(&self) -> Result<Option<BasePath>> {
        self.run.base.base_path()
    }
}

impl Cli {
    /// Build a runner based on `RunCmd`.
    /// This function is same as `create_runner` in `SubstrateCli`, but it just uses for `RunCmd`.
    pub fn create_runner_for_run_cmd(&self, command: &RunCmd) -> Result<Runner<Self>> {
        let tokio_runtime = build_runtime()?;
        // a hacky way to set network type directly.
        let is_dev = self.is_dev()?;
        let chain_id = self.chain_id(is_dev)?;
        let chain_spec = self.load_spec(&chain_id)?;
        match chain_spec.id() {
            "dev" | "local_testnet" => set_chain_network_type(ChainNetworkType::Dev),
            // TODO add mainnet and testnet
            _ => set_chain_network_type(ChainNetworkType::Testnet),
        }

        // we use our custom configuration (`Self`) to replace `RunCmd`'s configuration.
        let config = CliConfiguration::<Self>::create_configuration(
            self,
            self,
            tokio_runtime.handle().clone(),
        )?;

        command
            .base
            .init(&Self::support_url(), &Self::impl_version(), |_, _| {}, &config)?;
        Runner::new(config, tokio_runtime)
    }
}
