use bip39::{Language, Mnemonic, Seed};
use tiny_hderive::bip32::ExtendedPrivKey;
// Substrate
use sp_core::{crypto::DEV_PHRASE, ecdsa, hashing::keccak_256, Pair, Public, H160, H256};
// Local
use primitives_core::AccountId;
use runtime_common::{AuraId, GrandpaId};

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

// Child key at derivation path: m/44'/60'/0'/0/{index}
const DEFAULT_DERIVATION_PATH_PREFIX: &str = "m/44'/60'/0'/0/";

fn mnemonic_to_seed(mnemonic: &str) -> Option<Seed> {
    Mnemonic::from_phrase(mnemonic, Language::English)
        .map(|x| Seed::new(&x, ""))
        .ok()
}

fn derive_bip44_pair_from_seed<TPair: Pair>(seed: &[u8], path: &str) -> Option<TPair> {
    ExtendedPrivKey::derive(seed, path)
        .ok()
        .and_then(|account| TPair::from_seed_slice(&account.secret()).ok())
}

/// Helper function to derive `num_accounts` child pairs from mnemonics
/// Substrate derive function cannot be used because the derivation is different than Ethereum's
/// https://substrate.dev/rustdocs/v2.0.0/src/sp_core/ecdsa.rs.html#460-470
pub fn derive_bip44_pairs_from_mnemonic<TPair: Pair>(
    mnemonic: &str,
    num_accounts: u32,
) -> Vec<TPair> {
    let seed = mnemonic_to_seed(mnemonic).expect("Wrong mnemonic provided");

    let mut childs = Vec::new();
    for i in 0..num_accounts {
        if let Some(child_pair) = derive_bip44_pair_from_seed(
            seed.as_bytes(),
            &format!("{}{}", DEFAULT_DERIVATION_PATH_PREFIX, i),
        ) {
            childs.push(child_pair);
        } else {
            log::error!("An error occurred while deriving key {} from parent", i)
        }
    }
    childs
}

/// Helper function to get an `AccountId` from an ECDSA Key Pair.
pub fn get_account_id_from_pair(pair: ecdsa::Pair) -> Option<AccountId> {
    let decompressed =
        libsecp256k1::PublicKey::parse_compressed(&pair.public().0).ok()?.serialize();

    Some(H160::from(H256::from(keccak_256(&decompressed[1..65]))).into())
}

/// Function to generate accounts given a mnemonic and a number of child accounts to be generated
/// Defaults to a default mnemonic if no mnemonic is supplied
pub fn generate_accounts(mnemonic: String, num_accounts: u32) -> Vec<AccountId> {
    let childs = derive_bip44_pairs_from_mnemonic::<ecdsa::Pair>(&mnemonic, num_accounts);
    log::debug!("Account Generation");
    childs
        .iter()
        .filter_map(|pair| {
            let account = get_account_id_from_pair(pair.clone());
            log::debug!(
                "private_key {} --------> Account {:x?}",
                sp_core::hexdisplay::HexDisplay::from(&pair.clone().seed()),
                account
            );
            account
        })
        .collect()
}

/// Creates a group of eth accounts for dev.
///
/// The first one is regarded as sudo key.
pub fn generate_dev_accounts(num_accounts: u32) -> (AccountId, Vec<AccountId>) {
    let accounts = generate_accounts(DEV_PHRASE.into(), num_accounts);
    (accounts[0], accounts)
}

pub fn alith_public() -> ecdsa::Public {
    let seed = mnemonic_to_seed(DEV_PHRASE).expect("Wrong mnemonic provided");
    derive_bip44_pair_from_seed::<ecdsa::Pair>(
        seed.as_bytes(),
        &format!("{}{}", DEFAULT_DERIVATION_PATH_PREFIX, 0),
    )
    .map(|pair| pair.public())
    .unwrap()
}

pub fn baltathar_pair() -> ecdsa::Pair {
    let seed = mnemonic_to_seed(DEV_PHRASE).expect("Wrong mnemonic provided");
    derive_bip44_pair_from_seed(
        seed.as_bytes(),
        &format!("{}{}", DEFAULT_DERIVATION_PATH_PREFIX, 1),
    )
    .unwrap()
}
