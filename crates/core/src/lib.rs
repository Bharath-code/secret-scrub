//! SecretScrub core: local redaction engine.
//!
//! Processes content entirely in-process. Never transmits artifacts.

mod cancel;
mod detect;
mod export;
mod format;
mod limits;
mod placeholder;
mod rulepack;
mod scrub;
mod structure;
mod types;
mod workspace;

pub use cancel::{CancelFlag, ProgressEvent};
pub use export::{atomic_write, ensure_not_source, write_safety_summary, ExportError};
pub use format::{format_from_path, ContentFormat};
pub use limits::WorkspaceLimits;
pub use rulepack::{RulePack, RULE_PACK_VERSION};
pub use scrub::{scrub, scrub_with_path, ScrubConfig, ScrubError, ScrubResult};
pub use types::{
    ExitCodeKind, FileReport, Finding, SafetyStatus, SafetySummary, StructureStatus,
    SummaryFinding, PRODUCT_VERSION,
};
pub use workspace::{
    export_workspace_tree, scrub_workspace, FileArtifact, FileInclusion, FileOutcome,
    WorkspaceError, WorkspaceResult,
};

/// Crate-level product version (semver of this release).
pub fn product_version() -> &'static str {
    PRODUCT_VERSION
}

/// Built-in rule pack version identifier.
pub fn rule_pack_version() -> &'static str {
    RULE_PACK_VERSION
}
