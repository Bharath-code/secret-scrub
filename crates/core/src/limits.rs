//! Workspace and input limits (hostile / oversized input bounds).

use serde::{Deserialize, Serialize};

/// Default and configurable limits for single-file and folder scrubs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceLimits {
    /// Max directory recursion depth from the workspace root (0 = root only).
    pub max_depth: usize,
    /// Max bytes per file (metadata size check before full read).
    pub max_file_size: u64,
    /// Max files included in one workspace scrub.
    pub max_file_count: usize,
    /// Max characters per line for line-oriented plain-text streaming path.
    pub max_line_length: usize,
    /// Soft work budget: max files * processed bytes estimate checks via cancel points.
    /// When total bytes read exceed this, remaining files are excluded as over budget.
    pub max_total_bytes: u64,
}

impl Default for WorkspaceLimits {
    fn default() -> Self {
        Self {
            max_depth: 8,
            max_file_size: 10 * 1024 * 1024, // 10 MiB
            max_file_count: 500,
            max_line_length: 1024 * 1024, // 1 MiB
            max_total_bytes: 50 * 1024 * 1024, // 50 MiB per workspace
        }
    }
}

impl WorkspaceLimits {
    pub fn for_tests() -> Self {
        Self {
            max_depth: 4,
            max_file_size: 64 * 1024,
            max_file_count: 20,
            max_line_length: 8 * 1024,
            max_total_bytes: 256 * 1024,
        }
    }
}
