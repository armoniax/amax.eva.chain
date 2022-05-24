use sha3::{Digest, Keccak256};
use sp_core::{ecdsa, Pair, Public, H160, H256};

use amax_eva_runtime::{AccountId, AuraId, GrandpaId};
use log::debug;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

/// Function to generate accounts given a mnemonic and a number of child accounts to be generated
/// Defaults to a default mnemonic if no mnemonic is supplied
pub fn generate_accounts(mnemonic: String, num_accounts: u32) -> Vec<AccountId> {
    let childs = derive_bip44_pairs_from_mnemonic::<ecdsa::Public>(&mnemonic, num_accounts);
    debug!("Account Generation");
    childs
        .iter()
        .filter_map(|par| {
            let account = get_account_id_from_pair(par.clone());
            debug!(
                "private_key {} --------> Account {:x?}",
                sp_core::hexdisplay::HexDisplay::from(&par.clone().seed()),
                account
            );
            account
        })
        .collect()
}
/// Helper function to derive `num_accounts` child pairs from mnemonics
/// Substrate derive function cannot be used because the derivation is different than Ethereum's
/// https://substrate.dev/rustdocs/v2.0.0/src/sp_core/ecdsa.rs.html#460-470
pub fn derive_bip44_pairs_from_mnemonic<TPublic: Public>(
    mnemonic: &str,
    num_accounts: u32,
) -> Vec<TPublic::Pair> {
    use bip39::{Language, Mnemonic, Seed};
    use tiny_hderive::bip32::ExtendedPrivKey;

    let seed = Mnemonic::from_phrase(mnemonic, Language::English)
        .map(|x| Seed::new(&x, ""))
        .expect("Wrong mnemonic provided");

    let mut childs = Vec::new();
    for i in 0..num_accounts {
        if let Some(child_pair) =
            ExtendedPrivKey::derive(seed.as_bytes(), format!("m/44'/60'/0'/0/{}", i).as_ref())
                .ok()
                .and_then(|account| TPublic::Pair::from_seed_slice(&account.secret()).ok())
        {
            childs.push(child_pair);
        } else {
            log::error!("An error ocurred while deriving key {} from parent", i)
        }
    }
    childs
}
/// Helper function to get an `AccountId` from an ECDSA Key Pair.
pub fn get_account_id_from_pair(pair: ecdsa::Pair) -> Option<AccountId> {
    let decompressed = libsecp256k1::PublicKey::parse_slice(
        &pair.public().0,
        Some(libsecp256k1::PublicKeyFormat::Compressed),
    )
    .ok()?
    .serialize();

    let mut m = [0u8; 64];
    m.copy_from_slice(&decompressed[1..65]);

    Some(H160::from(H256::from_slice(Keccak256::digest(&m).as_slice())).into())
}

pub const DEV_MNEMONIC: &str =
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk";

pub fn generate_dev_accounts(num_accounts: u32) -> Vec<AccountId> {
    generate_accounts(DEV_MNEMONIC.into(), num_accounts)
}

pub fn benchmark_pair() -> ecdsa::Pair {
    let v = derive_bip44_pairs_from_mnemonic::<ecdsa::Public>(DEV_MNEMONIC, 0);
    v[0].clone()
}
