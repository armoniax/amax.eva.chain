/// Eva chain spec.
pub mod eva;
/// Key helper functions.
pub mod key_helper;
/// Wall-e chain spec.
pub mod wall_e;

use std::{fs::File, path::PathBuf};
// Substrate
use sc_chain_spec::{ChainSpec, Properties};

/// Can be called for a `Configuration` to check if it is a configuration for
/// the `Eva` network.
pub trait IdentifyVariant {
    /// Returns `true` if this is a configuration for the `Eva` network.
    fn is_eva(&self) -> bool;

    /// Returns `true` if this is a configuration for the `Wall-e` network.
    fn is_wall_e(&self) -> bool;
}

impl IdentifyVariant for &str {
    fn is_eva(&self) -> bool {
        self.starts_with("eva")
    }

    fn is_wall_e(&self) -> bool {
        self.starts_with("wall-e") || self.starts_with("dev")
    }
}

impl IdentifyVariant for &dyn ChainSpec {
    fn is_eva(&self) -> bool {
        self.id().is_eva()
    }

    fn is_wall_e(&self) -> bool {
        self.id().is_wall_e()
    }
}

impl IdentifyVariant for Box<dyn ChainSpec> {
    fn is_eva(&self) -> bool {
        self.as_ref().is_eva()
    }

    fn is_wall_e(&self) -> bool {
        self.as_ref().is_wall_e()
    }
}

/// Runtime chain spec.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum RuntimeChainSpec {
    Eva,
    WallE,
    Unknown,
}

pub trait RuntimeChain {
    fn runtime(&self) -> RuntimeChainSpec;
}

impl RuntimeChain for &str {
    fn runtime(&self) -> RuntimeChainSpec {
        if self.is_eva() {
            RuntimeChainSpec::Eva
        } else if self.is_wall_e() {
            RuntimeChainSpec::WallE
        } else {
            RuntimeChainSpec::Unknown
        }
    }
}

impl RuntimeChain for &dyn ChainSpec {
    fn runtime(&self) -> RuntimeChainSpec {
        self.id().runtime()
    }
}

impl RuntimeChain for Box<dyn ChainSpec> {
    fn runtime(&self) -> RuntimeChainSpec {
        self.as_ref().runtime()
    }
}

impl RuntimeChain for ChainId {
    fn runtime(&self) -> RuntimeChainSpec {
        self.id.as_str().runtime()
    }
}

#[derive(serde::Deserialize)]
pub struct ChainId {
    pub id: String,
}
impl ChainId {
    /// Parse json file into a `ChainSpec`.
    pub fn from_json_file(path: PathBuf) -> Result<Self, String> {
        // We mmap the file into memory first, as this is *a lot* faster than using
        // `serde_json::from_reader`. See https://github.com/serde-rs/json/issues/160
        let file = File::open(&path)
            .map_err(|e| format!("Error opening spec file `{}`: {}", path.display(), e))?;

        // SAFETY: `mmap` is fundamentally unsafe since technically the file can change
        //         underneath us while it is mapped; in practice it's unlikely to be a problem
        let bytes = unsafe {
            memmap2::Mmap::map(&file)
                .map_err(|e| format!("Error mmaping spec file `{}`: {}", path.display(), e))?
        };
        let client_spec = serde_json::from_slice(&bytes)
            .map_err(|e| format!("Error parsing spec file: {}", e))?;
        Ok(client_spec)
    }
}

pub(crate) fn properties() -> Properties {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "AMAX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties
}
