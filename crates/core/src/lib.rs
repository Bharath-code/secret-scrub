//! SecretScrub core: local redaction engine.
//!
//! Processes content entirely in-process. Never transmits artifacts.

mod detect;
mod export;
mod placeholder;
mod rulepack;
mod scrub;
mod types;

pub use export::{atomic_write, ensure_not_source, write_safety_summary, ExportError};
pub use rulepack::{RULE_PACK_VERSION, RulePack};
pub use scrub::{scrub, ScrubConfig, ScrubError, ScrubResult};
pub use types::{
    Finding, SafetyStatus, StructureStatus, SummaryFinding, SafetySummary, PRODUCT_VERSION,
};

/// Crate-level product version (semver of this release).
pub fn product_version() -> &'static str {
    PRODUCT_VERSION
}

/// Built-in rule pack version identifier.
pub fn rule_pack_version() -> &'static str {
    RULE_PACK_VERSION
}
