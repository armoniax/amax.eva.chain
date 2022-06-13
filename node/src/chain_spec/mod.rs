// chain-spec
pub mod eva;
pub mod wall_e;

pub(crate) mod key_helper;

use sc_chain_spec::{ChainSpec, Properties};

/// Can be called for a `Configuration` to check if it is a configuration for
/// the `Eva` network.
pub trait IdentifyVariant {
    /// Returns `true` if this is a configuration for the `Eva` network.
    fn is_eva(&self) -> bool;

    /// Returns `true` if this is a configuration for the `Wall-e` network.
    fn is_wall_e(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
    fn is_eva(&self) -> bool {
        self.id().starts_with("eva")
    }

    fn is_wall_e(&self) -> bool {
        self.id().starts_with("wall_e")
    }
}

pub(crate) fn properties() -> Properties {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "AMAX".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties
}
