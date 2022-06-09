// Substrate
use sc_service::{ChainType, Properties};
// Local
use eva_runtime::{AuraId, GenesisConfig, GrandpaId, SS58Prefix, WASM_BINARY};
use primitives_core::AccountId;

use super::key_helper::{authority_keys_from_seed, generate_dev_accounts};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

fn properties() -> Properties {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "AMAX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), SS58Prefix::get().into());
    properties
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    let accounts = generate_dev_accounts(10);

    Ok(ChainSpec::from_genesis(
        // Name
        "Eva Development",
        // ID
        "eva_dev",
        ChainType::Development,
        move || {
            genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                accounts[0],
                // Pre-funded accounts
                vec![
                    accounts[0], // Alith
                    accounts[1], // Baltathar
                    accounts[2], // Charleth
                    accounts[3], // Dorothy
                    accounts[4],
                    accounts[5],
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        None,
        // Properties
        Some(properties()),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    let accounts = generate_dev_accounts(10);

    Ok(ChainSpec::from_genesis(
        // Name
        "Eva Local Testnet",
        // ID
        "eva_local_testnet",
        ChainType::Local,
        move || {
            genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
                // Sudo account
                accounts[0], // Alith
                // Pre-funded accounts
                vec![
                    accounts[0], // Alith
                    accounts[1], // Baltathar
                    accounts[2], // Charleth
                    accounts[3], // Dorothy
                    accounts[4],
                    accounts[5],
                    accounts[6],
                    accounts[7],
                    accounts[8],
                    accounts[9],
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        Some(properties()),
        // Extensions
        None,
    ))
}

/// Configure initial storage state for FRAME modules.
fn genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    use eva_runtime::{AuraConfig, BalancesConfig, GrandpaConfig, SudoConfig, SystemConfig};
    GenesisConfig {
        // System && Utility.
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(root_key),
        },
        // Monetary.
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 80.
            balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 80)).collect(),
        },
        transaction_payment: Default::default(),
        // Consesnsus.
        aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
        },
        // Evm compatibility.
        evm: Default::default(),
        ethereum: Default::default(),
        base_fee: Default::default(),
    }
}
