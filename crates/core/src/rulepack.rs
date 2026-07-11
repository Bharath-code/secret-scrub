//! Versioned built-in rule pack metadata.

/// Semantic version of the bundled detector pack.
pub const RULE_PACK_VERSION: &str = "0.1.0";

/// Identifier for the built-in pack (future: custom packs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RulePack {
    #[default]
    BuiltinV1,
}

impl RulePack {
    pub fn version(self) -> &'static str {
        match self {
            RulePack::BuiltinV1 => RULE_PACK_VERSION,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            RulePack::BuiltinV1 => "builtin",
        }
    }
}
