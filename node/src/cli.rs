use std::net::SocketAddr;

use sc_cli::{
    build_runtime, ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams,
    KeystoreParams, NetworkParams, OffchainWorkerParams, Result, Role, Runner, SharedParams,
    SubstrateCli,
};
use sc_service::{config::PrometheusConfig, BasePath, Configuration, TransactionPoolOptions};
use sc_telemetry::TelemetryEndpoints;

use crate::chain_spec::IdentifyVariant;

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

    /// Number of concurrent tracing tasks. Meant to be shared by both "debug" and "trace" modules.
    #[clap(long, default_value = "10")]
    pub ethapi_max_permits: u32,

    /// Duration (in seconds) after which the cache of `trace_filter` for a given block will be
    /// discarded.
    #[clap(long, default_value = "300")]
    pub ethapi_trace_cache_duration: u64,

    /// Maximum number of trace entries a single request of `trace_filter` is allowed to return.
    /// A request asking for more or an unbounded one going over this limit will both return an
    /// error.
    #[clap(long, default_value = "500")]
    pub ethapi_trace_max_count: u32,

    /// Enable EVM tracing module on a non-authority node.
    #[clap(
        long,
        use_value_delimiter = true,
        require_value_delimiter = true,
        multiple_values = true
    )]
    pub ethapi: Vec<crate::tracing::EthApiExt>,
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

static mut GLOBAL_CHAIN_SPEC: Option<Box<dyn ChainSpec>> = None;
pub(crate) fn set_chain_spec(chain_spec: Box<dyn ChainSpec>) {
    // this is safe, for this function should only be called in `load_spec`.
    unsafe {
        GLOBAL_CHAIN_SPEC = Some(chain_spec);
    }
}
#[allow(clippy::borrowed_box)]
pub(crate) fn get_chain_spec() -> Option<&'static Box<dyn ChainSpec>> {
    // this is safe, for this function is not written when called.
    unsafe { GLOBAL_CHAIN_SPEC.as_ref() }
}

impl DefaultConfigurationValues for Cli {
    fn p2p_listen_port() -> u16 {
        let chain_spec =
            get_chain_spec().expect("ChainSpec must be set before this function is called");
        if chain_spec.is_eva() {
            return 9922
        }
        if chain_spec.is_wall_e() {
            return 19922
        }
        unreachable!("All runtime type should be captured");
    }

    fn rpc_ws_listen_port() -> u16 {
        let chain_spec =
            get_chain_spec().expect("ChainSpec must be set before this function is called");
        if chain_spec.is_eva() {
            return 9944
        }
        if chain_spec.is_wall_e() {
            return 19944
        }
        unreachable!("All runtime type should be captured");
    }

    fn rpc_http_listen_port() -> u16 {
        let chain_spec =
            get_chain_spec().expect("ChainSpec must be set before this function is called");
        if chain_spec.is_eva() {
            return 9933
        }
        if chain_spec.is_wall_e() {
            return 19933
        }
        unreachable!("All runtime type should be captured");
    }

    fn prometheus_listen_port() -> u16 {
        9616
    }
}

/// A copy implementation for RunCmd, but the generic type `DCV` in `CliConfiguration<DCV>` is
/// `Cli`. Notice this implementation should be updated if the `CliConfiguration` implementation for
/// `RunCmd` has changed.
/// More exactly, the functions which are override by `sc_cli::RunCmd` for `CliConfiguration<DCV>`
/// should be all override by `RunCmd` for `CliConfiguration<Cli>`.
impl CliConfiguration<Cli> for RunCmd {
    fn shared_params(&self) -> &SharedParams {
        self.base.shared_params()
    }
    fn import_params(&self) -> Option<&ImportParams> {
        self.base.import_params()
    }
    fn network_params(&self) -> Option<&NetworkParams> {
        self.base.network_params()
    }
    fn keystore_params(&self) -> Option<&KeystoreParams> {
        self.base.keystore_params()
    }
    fn offchain_worker_params(&self) -> Option<&OffchainWorkerParams> {
        self.base.offchain_worker_params()
    }
    fn node_name(&self) -> Result<String> {
        self.base.node_name()
    }
    fn dev_key_seed(&self, is_dev: bool) -> Result<Option<String>> {
        self.base.dev_key_seed(is_dev)
    }
    fn telemetry_endpoints(
        &self,
        chain_spec: &Box<dyn ChainSpec>,
    ) -> Result<Option<TelemetryEndpoints>> {
        self.base.telemetry_endpoints(chain_spec)
    }
    fn role(&self, is_dev: bool) -> Result<Role> {
        self.base.role(is_dev)
    }
    fn force_authoring(&self) -> Result<bool> {
        self.base.force_authoring()
    }
    fn prometheus_config(
        &self,
        default_listen_port: u16,
        chain_spec: &Box<dyn ChainSpec>,
    ) -> Result<Option<PrometheusConfig>> {
        self.base.prometheus_config(default_listen_port, chain_spec)
    }
    fn disable_grandpa(&self) -> Result<bool> {
        self.base.disable_grandpa()
    }
    fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
        self.base.rpc_ws_max_connections()
    }

    fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
        self.base.rpc_cors(is_dev)
    }
    fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
        self.base.rpc_http(default_listen_port)
    }
    fn rpc_ipc(&self) -> Result<Option<String>> {
        self.base.rpc_ipc()
    }
    fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
        self.base.rpc_ws(default_listen_port)
    }
    fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
        self.base.rpc_methods()
    }
    fn rpc_max_payload(&self) -> Result<Option<usize>> {
        self.base.rpc_max_payload()
    }

    fn ws_max_out_buffer_capacity(&self) -> Result<Option<usize>> {
        self.base.ws_max_out_buffer_capacity()
    }
    fn transaction_pool(&self, is_dev: bool) -> Result<TransactionPoolOptions> {
        self.base.transaction_pool(is_dev)
    }
    fn max_runtime_instances(&self) -> Result<Option<usize>> {
        self.base.max_runtime_instances()
    }
    fn runtime_cache_size(&self) -> Result<u8> {
        self.base.runtime_cache_size()
    }
    fn base_path(&self) -> Result<Option<BasePath>> {
        self.base.base_path()
    }
}

impl Cli {
    /// Build a runner with the config that is generated from outside.
    /// This function is same as `create_runner` in `SubstrateCli`, but it generate the config from
    /// the function closure.
    pub fn create_runner_with_config<T: CliConfiguration<()>>(
        &self,
        command: &T,
        f: impl Fn(&Self, tokio::runtime::Handle) -> Result<Configuration>,
    ) -> Result<Runner<Self>> {
        let tokio_runtime = build_runtime()?;
        // we use outside function to generate config
        let config = f(self, tokio_runtime.handle().clone())?;

        command.init(&Self::support_url(), &Self::impl_version(), |_, _| {}, &config)?;
        Runner::new(config, tokio_runtime)
    }
}
