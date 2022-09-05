// Substrate
use sc_service::ChainType;
// Local
use eva_runtime::{
    constants::currency::UNITS, AuraId, GenesisConfig, GrandpaId, SessionKeys, WASM_BINARY,
};
use primitives_core::{AccountId, Balance};

use super::{
    key_helper::{authority_keys_from_seed, generate_dev_accounts},
    properties,
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

pub fn development_chain_spec() -> ChainSpec {
    let wasm_binary = WASM_BINARY.expect("wasm not available");

    // 0 Alith
    // 1 Baltathar
    // 2 Charleth
    // 3 Dorothy
    // ...
    let (sudo_key, accounts) = generate_dev_accounts(10);

    ChainSpec::from_genesis(
        // Name
        "Eva Development",
        // ID
        "eva-dev",
        ChainType::Development,
        move || {
            let endowed = accounts.clone().into_iter().map(|k| (k, 100000 * UNITS)).collect();
            let alice = authority_keys_from_seed("Alice");
            genesis(
                wasm_binary,
                // Sudo account, Alith
                sudo_key,
                // Pre-funded accounts
                endowed,
                // Initial PoA authorities
                vec![
                    // Alith with Alice
                    (accounts[0], alice.0, alice.1),
                ],
                // Technical committee members
                vec![accounts[0], accounts[1], accounts[2]],
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Fork ID
        None,
        // Properties
        Some(properties()),
        // Extensions
        None,
    )
}

pub fn local_testnet_chain_spec() -> ChainSpec {
    let wasm_binary = WASM_BINARY.expect("wasm not available");

    // 0 Alith
    // 1 Baltathar
    // 2 Charleth
    // 3 Dorothy
    // ...
    let (sudo_key, accounts) = generate_dev_accounts(10);

    ChainSpec::from_genesis(
        // Name
        "Eva Local Testnet",
        // ID
        "eva-local",
        ChainType::Local,
        move || {
            let endowed = accounts.clone().into_iter().map(|k| (k, 100000 * UNITS)).collect();
            let alice = authority_keys_from_seed("Alice");
            let bob = authority_keys_from_seed("Bob");
            genesis(
                wasm_binary,
                // Sudo account, Alith
                sudo_key,
                // Pre-funded accounts
                endowed,
                // Initial PoA authorities
                vec![
                    // Alith with Alice
                    (accounts[0], alice.0, alice.1),
                    // Baltathar with Bob
                    (accounts[1], bob.0, bob.1),
                ],
                // Technical committee members
                vec![accounts[0], accounts[1], accounts[2]],
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Fork ID
        None,
        // Properties
        Some(properties()),
        // Extensions
        None,
    )
}

fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
    SessionKeys { aura, grandpa }
}

/// Configure initial storage state for FRAME modules.
fn genesis(
    wasm_binary: &[u8],
    sudo_key: AccountId,
    endowed: Vec<(AccountId, Balance)>,
    initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
    technical_committee: Vec<AccountId>,
) -> GenesisConfig {
    use eva_runtime::{
        AuthoritiesConfig, BalancesConfig, SessionConfig, SudoConfig, SystemConfig,
        TechnicalCommitteeConfig,
    };
    GenesisConfig {
        // System && Utility.
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(sudo_key),
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
        // Governance.
        technical_committee: TechnicalCommitteeConfig {
            members: technical_committee,
            phantom: Default::default(),
        },
        technical_committee_membership: Default::default(),
        // Evm compatibility.
        evm: Default::default(),
        ethereum: Default::default(),
        base_fee: Default::default(),
    }
}
