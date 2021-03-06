// Substrate
use sc_service::ChainType;
// Local
use eva_runtime::{AuraId, GenesisConfig, GrandpaId, SessionKeys, WASM_BINARY};
use eva_runtime_constants::currency::UNITS;
use primitives_core::{AccountId, Balance};

use super::key_helper::{authority_keys_from_seed, generate_dev_accounts};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // 0 Alith
    // 1 Baltathar
    // 2 Charleth
    // 3 Dorothy
    let accounts = generate_dev_accounts(10);

    Ok(ChainSpec::from_genesis(
        // Name
        "Eva Development",
        // ID
        "eva_dev",
        ChainType::Development,
        move || {
            let endowed = accounts.clone().into_iter().map(|k| (k, 100000 * UNITS)).collect();
            let alice = authority_keys_from_seed("Alice");
            genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    // Alith with Alice
                    (accounts[0], alice.0, alice.1),
                ],
                // Sudo account
                accounts[0],
                // Pre-funded accounts
                endowed,
                vec![accounts[0], accounts[1], accounts[2]],
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
        Some(super::properties()),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // 0 Alith
    // 1 Baltathar
    // 2 Charleth
    // 3 Dorothy
    let accounts = generate_dev_accounts(10);

    Ok(ChainSpec::from_genesis(
        // Name
        "Eva Local Testnet",
        // ID
        "eva_local_testnet",
        ChainType::Local,
        move || {
            let endowed = accounts.clone().into_iter().map(|k| (k, 100000 * UNITS)).collect();
            let alice = authority_keys_from_seed("Alice");
            let bob = authority_keys_from_seed("Alice");
            genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    // Alith with Alice
                    (accounts[0], alice.0, alice.1),
                    // Baltathar with Bob
                    (accounts[1], bob.0, bob.1),
                ],
                // Sudo account
                accounts[0], // Alith
                // Pre-funded accounts
                endowed,
                vec![accounts[0], accounts[1], accounts[2]],
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Fork ID.
        None,
        // Properties
        Some(super::properties()),
        // Extensions
        None,
    ))
}

fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
    SessionKeys { aura, grandpa }
}

/// Configure initial storage state for FRAME modules.
fn genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed: Vec<(AccountId, Balance)>,
    technical_committee: Vec<AccountId>,
) -> GenesisConfig {
    use eva_runtime::{
        AuthoritiesConfig, BalancesConfig, BaseFeeConfig, SessionConfig, SudoConfig, SystemConfig,
        TechnicalCommitteeConfig,
    };
    GenesisConfig {
        // System && Utility.
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        // Monetary.
        balances: BalancesConfig { balances: endowed },
        transaction_payment: Default::default(),
        // Consesnsus.
        aura: Default::default(),
        grandpa: Default::default(),
        session: SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| (x.0, x.0, session_keys(x.1.clone(), x.2.clone())))
                .collect::<Vec<_>>(),
        },
        authorities: AuthoritiesConfig {
            keys: initial_authorities.iter().map(|x| (x.0)).collect::<Vec<_>>(),
        },
        technical_committee: TechnicalCommitteeConfig {
            members: technical_committee,
            phantom: Default::default(),
        },
        technical_committee_membership: Default::default(),
        // Evm compatibility.
        evm: Default::default(),
        ethereum: Default::default(),
        base_fee: {
            let mut d = BaseFeeConfig::default();
            d.is_active = false;
            d
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(root_key),
        },
    }
}
