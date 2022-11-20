// Substrate
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use sc_service::DatabaseSource;
// Frontier
use fc_db::frontier_database_dir;

use crate::{
    chain_spec::{self, ChainId, RuntimeChain, RuntimeChainSpec},
    cli::{Cli, Subcommand},
    service::{self, db_config_dir},
};

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Armonia Eva Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/armoniax/amax.eva.chain/issues/new".into()
    }

    fn copyright_start_year() -> i32 {
        2022
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn ChainSpec>, String> {
        // a hacky way to set network type directly.
        crate::cli::set_chain_spec(id.runtime());
        Ok(match id {
            "" | "dev" | "wall-e-dev" => Box::new(chain_spec::wall_e::development_chain_spec()),
            "wall-e-local" => Box::new(chain_spec::wall_e::local_testnet_chain_spec()),
            "eva-dev" => Box::new(chain_spec::eva::development_chain_spec()),
            "eva-local" => Box::new(chain_spec::eva::local_testnet_chain_spec()),
            path => {
                // reset the global chain-spec again
                let chain_id = ChainId::from_json_file(path.into())?;
                crate::cli::set_chain_spec(chain_id.runtime());
                Box::new(chain_spec::wall_e::ChainSpec::from_json_file(path.into())?)
            },
        })
    }

    fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        match chain_spec.runtime() {
            RuntimeChainSpec::Eva => &eva_runtime::VERSION,
            RuntimeChainSpec::WallE => &wall_e_runtime::VERSION,
            RuntimeChainSpec::Unknown => panic!("Unknown chain spec"),
        }
    }
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        },
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager) =
                    service::new_chain_ops(&mut config, &cli)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        },
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager) = service::new_chain_ops(&mut config, &cli)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        },
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager) = service::new_chain_ops(&mut config, &cli)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        },
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager) =
                    service::new_chain_ops(&mut config, &cli)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        },
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| {
                // Remove Frontier offchain db
                let db_config_dir = db_config_dir(&config);
                let frontier_database_config = match config.database {
                    DatabaseSource::RocksDb { .. } => DatabaseSource::RocksDb {
                        path: frontier_database_dir(&db_config_dir, "db"),
                        cache_size: 0,
                    },
                    DatabaseSource::ParityDb { .. } => DatabaseSource::ParityDb {
                        path: frontier_database_dir(&db_config_dir, "paritydb"),
                    },
                    _ => {
                        return Err(format!("Cannot purge `{:?}` database", config.database).into())
                    },
                };
                cmd.run(frontier_database_config)?;
                cmd.run(config.database)
            })
        },
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, backend, _, task_manager) = service::new_chain_ops(&mut config, &cli)?;
                let aux_revert = Box::new(move |client, _, blocks| {
                    sc_finality_grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
            })
        },
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            use crate::{
                benchmarking::{
                    inherent_benchmark_data, ExistentialDepositProvider, RemarkBuilder,
                    TransferKeepAliveBuilder,
                },
                chain_spec::key_helper::alith_public,
                client::{EvaExecutor, WallEExecutor},
            };
            use frame_benchmarking_cli::{
                BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE,
            };

            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            // This switch needs to be in the client, since the client decides
            // which sub-commands it wants to support.
            match cmd {
                BenchmarkCmd::Pallet(cmd) => match chain_spec.runtime() {
                    RuntimeChainSpec::Eva => {
                        runner.sync_run(|config| cmd.run::<eva_runtime::Block, EvaExecutor>(config))
                    },
                    RuntimeChainSpec::WallE => runner
                        .sync_run(|config| cmd.run::<wall_e_runtime::Block, WallEExecutor>(config)),
                    RuntimeChainSpec::Unknown => panic!("Unknown chain spec"),
                },
                BenchmarkCmd::Block(cmd) => runner.sync_run(|mut config| {
                    let (client, _, _, _) = service::new_chain_ops(&mut config, &cli)?;
                    unwrap_client!(client, cmd.run(client.clone()))
                }),
                BenchmarkCmd::Storage(cmd) => runner.sync_run(|mut config| {
                    let (client, backend, _, _) = service::new_chain_ops(&mut config, &cli)?;
                    let db = backend.expose_db();
                    let storage = backend.expose_storage();
                    unwrap_client!(client, cmd.run(config, client.clone(), db, storage))
                }),
                BenchmarkCmd::Machine(cmd) => {
                    runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()))
                },
                BenchmarkCmd::Overhead(cmd) => runner.sync_run(|mut config| {
                    let (client, _, _, _) = service::new_chain_ops(&mut config, &cli)?;
                    let ext_builder = RemarkBuilder::new(client.clone());
                    unwrap_client!(
                        client,
                        cmd.run(config, client.clone(), inherent_benchmark_data()?, &ext_builder)
                    )
                }),
                BenchmarkCmd::Extrinsic(cmd) => runner.sync_run(|mut config| {
                    let (client, _, _, _) = service::new_chain_ops(&mut config, &cli)?;
                    // Register the *Remark* and *TKA* builders.
                    let ext_factory = ExtrinsicFactory(vec![
                        Box::new(RemarkBuilder::new(client.clone())),
                        Box::new(TransferKeepAliveBuilder::new(
                            client.clone(),
                            alith_public().into(),
                            client.existential_deposit(),
                        )),
                    ]);
                    unwrap_client!(
                        client,
                        cmd.run(client.clone(), inherent_benchmark_data()?, &ext_factory)
                    )
                }),
            }
        },
        #[cfg(not(feature = "runtime-benchmarks"))]
        Some(Subcommand::Benchmark) => Err("Benchmarking wasn't enabled when building the node. \
        	You can enable it with `--features runtime-benchmarks`."
            .into()),
        #[cfg(feature = "try-runtime")]
        Some(Subcommand::TryRuntime(cmd)) => {
            use crate::{
                chain_spec::IdentifyVariant,
                client::{EvaExecutor, WallEExecutor},
            };

            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;
            let registry = &runner.config().prometheus_config.as_ref().map(|cfg| &cfg.registry);
            let task_manager =
                sc_service::TaskManager::new(runner.config().tokio_handle.clone(), *registry)
                    .map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;
            if chain_spec.is_eva() {
                return runner.async_run(|config| {
                    Ok((cmd.run::<eva_runtime::Block, EvaExecutor>(config), task_manager))
                })
            }
            if chain_spec.is_wall_e() {
                return runner.async_run(|config| {
                    Ok((cmd.run::<wall_e_runtime::Block, WallEExecutor>(config), task_manager))
                })
            }
            Err("All runtime type should be captured".into())
        },
        #[cfg(not(feature = "try-runtime"))]
        Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
        	You can enable it with `--features try-runtime`."
            .into()),
        None => {
            let runner = cli.create_runner_with_config(&cli.run.base, |cli, tokio_handle| {
                // note it's `cli.run` not `cli.run.base` here, for `cli.run` is implemented by
                // `CliConfiguration<Cli>`, for `CliConfiguration<()>`
                SubstrateCli::create_configuration(cli, &cli.run, tokio_handle)
            })?;
            runner.run_node_until_exit(|config| async move {
                service::build_full(config, &cli).map_err(Into::into)
            })
        },
    }
}
