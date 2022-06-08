// chain-spec
pub mod eva;
pub mod wall_e;

mod key_helper;

use sc_chain_spec::ChainSpec;

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
        self.id().starts_with("wall-e")
    }
}
