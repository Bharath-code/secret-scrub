//! Cooperative cancellation for workspace scrubs and long scans.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared cancel flag. Default is never cancelled.
#[derive(Debug, Clone, Default)]
pub struct CancelFlag {
    inner: Arc<AtomicBool>,
}

impl CancelFlag {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.inner.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }
}

/// Structured progress events for CLI/UI adapters (no secret material).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressEvent {
    WorkspaceStarted { root: String },
    FileStarted { path: String },
    FileFinished { path: String, included: bool },
    WorkspaceFinished { file_count: usize },
    Cancelled,
    LimitHit { kind: String, detail: String },
}

