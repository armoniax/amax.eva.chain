// use frame_system::Account;
use hex_literal::hex;
use sp_core::crypto::UncheckedInto;

// Substrate
use sc_service::ChainType;
// Local
use primitives_core::{AccountId, Balance};
use wall_e_runtime::{
    constants::currency::UNITS, AuraId, GenesisConfig, GrandpaId, SessionKeys, WASM_BINARY,
};

use super::{
    key_helper::{authority_keys_from_seed, generate_dev_accounts},
    properties,
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

pub fn testnet_endowed(accounts: &[AccountId]) -> Vec<(AccountId, Balance)> {
    // Configure endowed accounts with initial balance of 1 << (4+18).
    accounts.iter().map(|a| (*a, 20_0000 * UNITS)).collect()
}

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
        "Wall-e Development",
        // ID
        "wall-e-dev",
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
                // Technical committee memebers
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

    let (sudo_key, accounts) = generate_dev_accounts(10);

    ChainSpec::from_genesis(
        // Name
        "Wall-e Testnet Local",
        // ID
        "wall-e-local",
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

pub fn testnet_chain_spec() -> ChainSpec {
    let wasm_binary = WASM_BINARY.expect("wasm not available");
    ChainSpec::from_genesis(
        // Name
        "Wall-e Testnet",
        // ID
        "wall-e",
        ChainType::Live,
        move || {
            let accounts = vec![
                hex!["8315b078Cb5c268739c2AD9D5c29607023F44666"].into(), /* m/44'/60'/0'/0/0 -
                                                                          * sudo key */
                hex!["88033FBB419ceB06eA06ED628F1BC37aB506CD9A"].into(), // m/44'/60'/0'/0/1 - v1
                hex!["957099192d41787F30bF6fc81abe7ceDe81Bea60"].into(), // m/44'/60'/0'/0/2 - v2
                hex!["b655EFBe8006ea7439481a5Da8ebcbEEeC3d8D1f"].into(), // m/44'/60'/0'/0/3 - v3
                hex!["7a894fBA436A071240EBcCc6Bdd387AF32447A8d"].into(), // m/44'/60'/0'/0/4 - v4
                hex!["8e12f8b301DBCb5F5d20655CA5f9F9AB8B4Bc8c4"].into()  // m/44'/60'/0'/0/5 - v5
            ];
            let sudo_key: AccountId = accounts[0];
            let endowed = testnet_endowed(&accounts);
            let auras: Vec<AuraId> = vec![
                hex!["dac617ec3997e476f4a3939e5570087e19ec645328e0aa69fa13c6f2d1c50726"]
                    .unchecked_into(), //0
                hex!["2ea783defec0060c66c9cacf6a0fd614ba2505735d46f42b0b45460f49b3640c"]
                    .unchecked_into(), //1 - v1
                hex!["bac08021b2721d91e4933597c5da524aa79e186768259753230e4d398c18bd7e"]
                    .unchecked_into(), //2 - v2
                hex!["20e2418db201f89473764f669e099f84a02f745c07051fa9dd422c395f55b372"]
                    .unchecked_into(), //3 - v3
                hex!["e050d82e84bfae6573016df009beb33417b11e1e7013a7d839ba305c14af8c73"]
                    .unchecked_into(), //4 - v4
                hex!["1a719769512e4ddb4806e79fee66e4d788374b829b4d4de360f6e9fc2c5cb51e"]
                    .unchecked_into(), //5 - v5
            ];

            let grandpas: Vec<GrandpaId> = vec![
                hex!["8c6a4ba2cbab3254f0cab38c5b898689264766ac12ae4589e988ef9e81d28c9d"]
                    .unchecked_into(), //0
                hex!["defefca0316265154547dce658fc3772f9b2cc469ec0082991a0b2ea203aca5b"]
                    .unchecked_into(), //1 - v1
                hex!["e8a3237372c738cdf09d2ec28ead04064ec60cd93cadd5f295083664591488cc"]
                    .unchecked_into(), //2 - v2
                hex!["24166cdbadb5ed38409dfb86179d01836ce8e2f1942a52f82865da3ed7f5b063"]
                    .unchecked_into(), //3 - v3
                hex!["6c075fdde3f5e1f0f2fdd3bdb08ba4803e0e274711c062070cbce65009cee966"]
                    .unchecked_into(), //4 - v4
                hex!["f3767106ad455f2cfcd5419aaa9668ce16fb719f4b7ebcf140cffab3f6a34911"]
                    .unchecked_into(), //5 - v5
            ];

            genesis(
                wasm_binary,
                sudo_key, // Sudo account
                endowed,  // Pre-funded accounts
                //initial validators
                vec![
                    (accounts[1], auras[1].clone(), grandpas[1].clone()),
                    (accounts[2], auras[2].clone(), grandpas[2].clone()),
                    (accounts[3], auras[3].clone(), grandpas[3].clone()),
                ],
                // Technical committee members: 5
                accounts,
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
    technical_committee: Vec<AccountId>, /*,
                                         xchain_committee: Vec<AccountId> */
) -> GenesisConfig {
    use wall_e_runtime::{
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
        // xchain_committee: XchainCommitteeConfig {
        //     members: xchain_committee,
        //     phantom: Default::default(),
        // },
        // xchain_committee_membership: Default::default(),
        // Evm compatibility.
        evm: Default::default(),
        ethereum: Default::default(),
        base_fee: Default::default(),
    }
}
