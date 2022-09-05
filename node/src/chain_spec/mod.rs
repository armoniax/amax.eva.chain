/// Eva chain spec.
pub mod eva;
/// Key helper functions.
pub mod key_helper;
/// Wall-e chain spec.
pub mod wall_e;

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

pub(crate) fn properties() -> Properties {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "AMAX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties
}
