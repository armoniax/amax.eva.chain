use sc_service::{ChainType, Properties};

use amax_eva_runtime::{AccountId, AuraId, GrandpaId, SS58Prefix, WASM_BINARY};
// genesis config
use amax_eva_runtime::{
    AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, SudoConfig, SystemConfig,
};

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

    let accounts = crate::key_helper::generate_dev_accounts(10);

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![crate::key_helper::authority_keys_from_seed("Alice")],
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

    let accounts = crate::key_helper::generate_dev_accounts(10);

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    crate::key_helper::authority_keys_from_seed("Alice"),
                    crate::key_helper::authority_keys_from_seed("Bob"),
                ],
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
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(root_key),
        },
        transaction_payment: Default::default(),
        evm: Default::default(),
        ethereum: Default::default(),
        base_fee: Default::default(),
    }
}
