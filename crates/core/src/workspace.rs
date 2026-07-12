//! Multi-file workspace enumeration and correlated scrub.

use crate::cancel::{CancelFlag, ProgressEvent};
use crate::detect::find_candidates;
use crate::export::ExportError;
use crate::format::{format_from_path, looks_binary, ContentFormat};
use crate::limits::WorkspaceLimits;
use crate::placeholder::PlaceholderAllocator;
use crate::rulepack::RulePack;
use crate::structure::{findings_from_counts, scrub_structured};
use crate::types::{Finding, SafetyStatus, StructureStatus, PRODUCT_VERSION};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("cancelled")]
    Cancelled,
    #[error("not a directory: {0}")]
    NotADirectory(PathBuf),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("export error: {0}")]
    Export(#[from] ExportError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileInclusion {
    Included,
    Excluded,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileOutcome {
    pub path: String,
    pub inclusion: FileInclusion,
    pub reason: Option<String>,
    pub structure_status: StructureStatus,
    pub safety_status: SafetyStatus,
    pub findings_count: usize,
}

#[derive(Debug, Clone)]
pub struct FileArtifact {
    pub relative_path: PathBuf,
    pub outcome: FileOutcome,
    /// Safe text when included (or redacted fallback when review-required but text produced).
    pub text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceResult {
    pub root: PathBuf,
    pub files: Vec<FileArtifact>,
    pub findings: Vec<Finding>,
    pub safety_status: SafetyStatus,
    pub structure_status: StructureStatus,
    pub rule_pack_version: String,
    pub product_version: String,
    pub cancelled: bool,
}

impl WorkspaceResult {
    /// True when every candidate was unsupported/excluded and nothing useful was produced.
    pub fn is_fully_unsupported(&self) -> bool {
        !self.files.is_empty()
            && self.files.iter().all(|f| {
                matches!(
                    f.outcome.inclusion,
                    FileInclusion::Unsupported | FileInclusion::Excluded
                ) && f.text.is_none()
            })
    }
}

/// Scrub a directory as one correlated workspace.
pub fn scrub_workspace(
    root: &Path,
    _session_seed: u64,
    rule_pack: RulePack,
    limits: &WorkspaceLimits,
    cancel: &CancelFlag,
    mut progress: Option<&mut dyn FnMut(ProgressEvent)>,
) -> Result<WorkspaceResult, WorkspaceError> {
    if !root.is_dir() {
        return Err(WorkspaceError::NotADirectory(root.to_path_buf()));
    }

    emit(
        &mut progress,
        ProgressEvent::WorkspaceStarted {
            root: root.display().to_string(),
        },
    );

    let mut allocator = PlaceholderAllocator::new();
    let mut total_counts: HashMap<(String, String), usize> = HashMap::new();
    let mut artifacts: Vec<FileArtifact> = Vec::new();
    let mut bytes_read: u64 = 0;
    let mut included_count = 0usize;

    let entries = collect_entries(root, limits)?;

    for entry in entries {
        if cancel.is_cancelled() {
            emit(&mut progress, ProgressEvent::Cancelled);
            return Ok(partial_cancelled(
                root,
                artifacts,
                &allocator,
                &total_counts,
                rule_pack,
            ));
        }

        // Fail closed: a path that can't be relativized against root must
        // never be joined onto the staging dir at export time (that would
        // write outside it). Exclude it explicitly instead of falling back
        // to the absolute path. Unreachable via WalkDir today, but this is
        // the last line of defense if that ever changes.
        let rel = match entry.strip_prefix(root) {
            Ok(r) => r.to_path_buf(),
            Err(_) => {
                artifacts.push(FileArtifact {
                    relative_path: entry.clone(),
                    outcome: FileOutcome {
                        path: entry.to_string_lossy().replace('\\', "/"),
                        inclusion: FileInclusion::Excluded,
                        reason: Some("path outside workspace root".into()),
                        structure_status: StructureStatus::NotApplicable,
                        safety_status: SafetyStatus::ReviewRequired,
                        findings_count: 0,
                    },
                    text: None,
                });
                continue;
            }
        };
        let rel_str = rel.to_string_lossy().replace('\\', "/");

        emit(
            &mut progress,
            ProgressEvent::FileStarted {
                path: rel_str.clone(),
            },
        );

        let meta = match fs::symlink_metadata(&entry) {
            Ok(m) => m,
            Err(e) => {
                artifacts.push(excluded_file(
                    &rel,
                    FileInclusion::Excluded,
                    format!("stat failed: {e}"),
                ));
                continue;
            }
        };

        if meta.file_type().is_symlink() {
            artifacts.push(excluded_file(
                &rel,
                FileInclusion::Excluded,
                "symlink skipped (fail closed; not followed)".into(),
            ));
            emit(
                &mut progress,
                ProgressEvent::FileFinished {
                    path: rel_str,
                    included: false,
                },
            );
            continue;
        }

        if !meta.is_file() {
            continue;
        }

        if included_count >= limits.max_file_count {
            emit(
                &mut progress,
                ProgressEvent::LimitHit {
                    kind: "max_file_count".into(),
                    detail: format!("{}", limits.max_file_count),
                },
            );
            artifacts.push(excluded_file(
                &rel,
                FileInclusion::Excluded,
                format!("max file count {}", limits.max_file_count),
            ));
            continue;
        }

        let size = meta.len();
        if size > limits.max_file_size {
            artifacts.push(excluded_file(
                &rel,
                FileInclusion::Excluded,
                format!(
                    "file size {size} exceeds max_file_size {}",
                    limits.max_file_size
                ),
            ));
            emit(
                &mut progress,
                ProgressEvent::FileFinished {
                    path: rel_str,
                    included: false,
                },
            );
            continue;
        }

        if bytes_read.saturating_add(size) > limits.max_total_bytes {
            artifacts.push(excluded_file(
                &rel,
                FileInclusion::Excluded,
                format!(
                    "workspace total bytes would exceed max_total_bytes {}",
                    limits.max_total_bytes
                ),
            ));
            emit(
                &mut progress,
                ProgressEvent::FileFinished {
                    path: rel_str,
                    included: false,
                },
            );
            continue;
        }

        let bytes = match fs::read(&entry) {
            Ok(b) => b,
            Err(e) => {
                artifacts.push(excluded_file(
                    &rel,
                    FileInclusion::Excluded,
                    format!("read failed: {e}"),
                ));
                continue;
            }
        };
        bytes_read = bytes_read.saturating_add(bytes.len() as u64);

        let mut format = format_from_path(&entry);
        if looks_binary(&bytes) {
            format = ContentFormat::BinaryUnsupported;
        }

        if format == ContentFormat::BinaryUnsupported || format == ContentFormat::TomlUnsupported {
            let note = if format == ContentFormat::TomlUnsupported {
                "TOML unsupported in private beta"
            } else {
                "binary or non-text content unsupported"
            };
            artifacts.push(FileArtifact {
                relative_path: rel.clone(),
                outcome: FileOutcome {
                    path: rel_str.clone(),
                    inclusion: FileInclusion::Unsupported,
                    reason: Some(note.into()),
                    structure_status: StructureStatus::Unsupported,
                    safety_status: SafetyStatus::ReviewRequired,
                    findings_count: 0,
                },
                text: None,
            });
            emit(
                &mut progress,
                ProgressEvent::FileFinished {
                    path: rel_str,
                    included: false,
                },
            );
            continue;
        }

        // Line-length guard for plain text / env
        if matches!(format, ContentFormat::PlainText | ContentFormat::Env) {
            if let Some(bad) = bytes.split(|&b| b == b'\n').find(|line| {
                line.len() > limits.max_line_length
            }) {
                let _ = bad;
                artifacts.push(excluded_file(
                    &rel,
                    FileInclusion::Excluded,
                    format!(
                        "line exceeds max_line_length {}",
                        limits.max_line_length
                    ),
                ));
                emit(
                    &mut progress,
                    ProgressEvent::FileFinished {
                        path: rel_str,
                        included: false,
                    },
                );
                continue;
            }
        }

        let content = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                artifacts.push(FileArtifact {
                    relative_path: rel.clone(),
                    outcome: FileOutcome {
                        path: rel_str.clone(),
                        inclusion: FileInclusion::Unsupported,
                        reason: Some("invalid UTF-8".into()),
                        structure_status: StructureStatus::Unsupported,
                        safety_status: SafetyStatus::ReviewRequired,
                        findings_count: 0,
                    },
                    text: None,
                });
                emit(
                    &mut progress,
                    ProgressEvent::FileFinished {
                        path: rel_str,
                        included: false,
                    },
                );
                continue;
            }
        };

        if content.is_empty() {
            // Empty files are trivially safe; copy them through so the
            // exported bundle keeps the original tree shape.
            included_count += 1;
            let (safety_status, reason) =
                with_path_review(&rel_str, SafetyStatus::SafeCopyReady, Some("empty file".into()));
            artifacts.push(FileArtifact {
                relative_path: rel,
                outcome: FileOutcome {
                    path: rel_str.clone(),
                    inclusion: FileInclusion::Included,
                    reason,
                    structure_status: StructureStatus::NotApplicable,
                    safety_status,
                    findings_count: 0,
                },
                text: Some(String::new()),
            });
            emit(
                &mut progress,
                ProgressEvent::FileFinished {
                    path: rel_str,
                    included: true,
                },
            );
            continue;
        }

        let structured = scrub_structured(&content, format, &mut allocator);
        for (k, v) in &structured.counts {
            *total_counts.entry(k.clone()).or_insert(0) += *v;
        }

        let file_findings = findings_from_counts(structured.counts.clone(), &allocator);

        let inclusion = if structured.structure_status == StructureStatus::Unsupported {
            FileInclusion::Unsupported
        } else {
            FileInclusion::Included
        };

        let (safety_status, reason) =
            with_path_review(&rel_str, structured.safety_status, structured.note);

        included_count += 1;
        artifacts.push(FileArtifact {
            relative_path: rel,
            outcome: FileOutcome {
                path: rel_str.clone(),
                inclusion,
                reason,
                structure_status: structured.structure_status,
                safety_status,
                findings_count: file_findings.len(),
            },
            text: Some(structured.text),
        });
        emit(
            &mut progress,
            ProgressEvent::FileFinished {
                path: rel_str,
                included: true,
            },
        );
    }

    let findings = findings_from_counts(total_counts, &allocator);
    let (safety_status, structure_status) = aggregate_status(&artifacts);

    emit(
        &mut progress,
        ProgressEvent::WorkspaceFinished {
            file_count: artifacts.len(),
        },
    );

    Ok(WorkspaceResult {
        root: root.to_path_buf(),
        files: artifacts,
        findings,
        safety_status,
        structure_status,
        rule_pack_version: rule_pack.version().to_string(),
        product_version: PRODUCT_VERSION.to_string(),
        cancelled: false,
    })
}

fn collect_entries(root: &Path, limits: &WorkspaceLimits) -> Result<Vec<PathBuf>, WorkspaceError> {
    let mut paths = Vec::new();
    // max_depth: 0 = root files only; N = root + N levels of nesting.
    let walker = WalkDir::new(root)
        .follow_links(false)
        .max_depth(limits.max_depth.saturating_add(1));

    for entry in walker {
        let entry = entry.map_err(|e| WorkspaceError::Io(std::io::Error::other(e.to_string())))?;
        let ft = entry.file_type();
        if ft.is_symlink() {
            // Include symlink paths so we can record exclusion (walkdir may list them as files).
            paths.push(entry.path().to_path_buf());
            continue;
        }
        if ft.is_file() {
            paths.push(entry.path().to_path_buf());
        }
    }
    paths.sort();
    Ok(paths)
}

/// File and directory *names* are never scanned as content (only bytes
/// inside a file are). This checks the relative path string itself
/// against the same detectors used on content, so an obviously sensitive
/// filename (an email address, an IP, a provider token pattern) forces
/// review instead of shipping unremarked.
fn path_review_note(rel_str: &str) -> Option<String> {
    if find_candidates(rel_str).is_empty() {
        None
    } else {
        Some(
            "file path itself matches a detector pattern; file names are not scrubbed, only contents \
             (see TRUST.md known detection limits)"
                .to_string(),
        )
    }
}

fn with_path_review(
    rel_str: &str,
    safety_status: SafetyStatus,
    reason: Option<String>,
) -> (SafetyStatus, Option<String>) {
    match path_review_note(rel_str) {
        None => (safety_status, reason),
        Some(note) => {
            let combined = match reason {
                Some(r) => format!("{r}; {note}"),
                None => note,
            };
            (SafetyStatus::ReviewRequired, Some(combined))
        }
    }
}

fn excluded_file(rel: &Path, inclusion: FileInclusion, reason: String) -> FileArtifact {
    let path = rel.to_string_lossy().replace('\\', "/");
    FileArtifact {
        relative_path: rel.to_path_buf(),
        outcome: FileOutcome {
            path,
            inclusion,
            reason: Some(reason),
            structure_status: StructureStatus::NotApplicable,
            safety_status: SafetyStatus::ReviewRequired,
            findings_count: 0,
        },
        text: None,
    }
}

fn aggregate_status(files: &[FileArtifact]) -> (SafetyStatus, StructureStatus) {
    let mut safety = SafetyStatus::SafeCopyReady;
    let mut structure = StructureStatus::NotApplicable;

    for f in files {
        if matches!(
            f.outcome.inclusion,
            FileInclusion::Excluded | FileInclusion::Unsupported
        ) || f.outcome.safety_status == SafetyStatus::ReviewRequired
        {
            safety = SafetyStatus::ReviewRequired;
        }
        match f.outcome.structure_status {
            StructureStatus::Invalid | StructureStatus::Unsupported => {
                structure = f.outcome.structure_status;
            }
            StructureStatus::Valid if structure == StructureStatus::NotApplicable => {
                structure = StructureStatus::Valid;
            }
            _ => {}
        }
    }

    if files.is_empty() {
        safety = SafetyStatus::ReviewRequired;
    }

    (safety, structure)
}

fn partial_cancelled(
    root: &Path,
    artifacts: Vec<FileArtifact>,
    allocator: &PlaceholderAllocator,
    counts: &HashMap<(String, String), usize>,
    rule_pack: RulePack,
) -> WorkspaceResult {
    let findings = findings_from_counts(counts.clone(), allocator);
    WorkspaceResult {
        root: root.to_path_buf(),
        files: artifacts,
        findings,
        safety_status: SafetyStatus::ReviewRequired,
        structure_status: StructureStatus::NotApplicable,
        rule_pack_version: rule_pack.version().to_string(),
        product_version: PRODUCT_VERSION.to_string(),
        cancelled: true,
    }
}

fn emit(progress: &mut Option<&mut dyn FnMut(ProgressEvent)>, event: ProgressEvent) {
    if let Some(cb) = progress {
        cb(event);
    }
}

/// Write included safe files into `dest_root` as a parallel tree.
/// Builds in a temp sibling dir and swaps in only on success; cancel or
/// failure never touches a pre-existing destination.
pub fn export_workspace_tree(
    result: &WorkspaceResult,
    dest_root: &Path,
    force: bool,
    cancel: &CancelFlag,
) -> Result<(), WorkspaceError> {
    if dest_root.exists() && !force {
        return Err(WorkspaceError::Export(ExportError::DestinationExists(
            dest_root.to_path_buf(),
        )));
    }

    // Build the full tree in a temp sibling dir first so a pre-existing
    // destination is only replaced after the export completes; cancel or
    // failure just drops the temp dir.
    let parent = dest_root
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let staging = tempfile::Builder::new()
        .prefix(".secretscrub-export-")
        .tempdir_in(parent)
        .map_err(WorkspaceError::Io)?;

    for file in &result.files {
        if cancel.is_cancelled() {
            return Err(WorkspaceError::Cancelled);
        }
        // Export any produced text (including review-required structured fallbacks).
        let Some(text) = &file.text else {
            continue;
        };
        let dest = staging.path().join(&file.relative_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, text)?;
    }

    // Re-check after the loop: with zero files the loop never observes the
    // flag, and a cancelled export must never replace the destination.
    if cancel.is_cancelled() {
        return Err(WorkspaceError::Cancelled);
    }

    // staging.keep() disables the TempDir's auto-cleanup-on-drop; from here
    // on every path (success or failure) must explicitly remove it.
    let staging_path = staging.keep();

    if !dest_root.exists() {
        if let Err(e) = fs::rename(&staging_path, dest_root) {
            let _ = fs::remove_dir_all(&staging_path);
            return Err(WorkspaceError::Io(e));
        }
        return Ok(());
    }

    // Force-swap over an existing destination: rename it aside first so
    // there is never a window where the destination is gone but the
    // replacement isn't in place yet. If the final swap fails, restore
    // the aside copy so the user never loses their previous export.
    let aside = aside_path(dest_root);
    if let Err(e) = fs::rename(dest_root, &aside) {
        let _ = fs::remove_dir_all(&staging_path);
        return Err(WorkspaceError::Io(e));
    }
    match fs::rename(&staging_path, dest_root) {
        Ok(()) => {
            if aside.is_dir() {
                let _ = fs::remove_dir_all(&aside);
            } else {
                let _ = fs::remove_file(&aside);
            }
            Ok(())
        }
        Err(e) => {
            let _ = fs::rename(&aside, dest_root);
            let _ = fs::remove_dir_all(&staging_path);
            Err(WorkspaceError::Io(e))
        }
    }
}

/// Sibling path to rename the old destination to during a force-swap.
/// Includes the current PID so concurrent exports in the same parent
/// directory don't collide.
fn aside_path(dest_root: &Path) -> PathBuf {
    let file_name = dest_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    dest_root.with_file_name(format!(
        ".secretscrub-export-aside-{}-{file_name}",
        std::process::id()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn correlates_across_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.log"), "k=AKIAIOSFODNN7EXAMPLE\n").unwrap();
        fs::write(dir.path().join("b.log"), "again=AKIAIOSFODNN7EXAMPLE\n").unwrap();
        let cancel = CancelFlag::new();
        let result = scrub_workspace(
            dir.path(),
            0,
            RulePack::BuiltinV1,
            &WorkspaceLimits::for_tests(),
            &cancel,
            None,
        )
        .unwrap();
        let texts: Vec<_> = result
            .files
            .iter()
            .filter_map(|f| f.text.as_ref())
            .collect();
        assert_eq!(texts.len(), 2);
        let ph = &result.findings[0].placeholder;
        assert!(texts[0].contains(ph));
        assert!(texts[1].contains(ph));
    }

    #[test]
    fn skips_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("real.log");
        fs::write(&target, "AKIAIOSFODNN7EXAMPLE\n").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, dir.path().join("link.log")).unwrap();
        }
        #[cfg(not(unix))]
        {
            return;
        }
        let cancel = CancelFlag::new();
        let result = scrub_workspace(
            dir.path(),
            0,
            RulePack::BuiltinV1,
            &WorkspaceLimits::for_tests(),
            &cancel,
            None,
        )
        .unwrap();
        assert!(result.files.iter().any(|f| {
            f.outcome.path.contains("link")
                && f.outcome.inclusion == FileInclusion::Excluded
        }));
    }

    #[test]
    fn cancel_mid_workspace() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..5 {
            let mut f = fs::File::create(dir.path().join(format!("f{i}.log"))).unwrap();
            writeln!(f, "AKIAIOSFODNN7EXAMPLE").unwrap();
        }
        let cancel = CancelFlag::new();
        let cancel2 = cancel.clone();
        let mut n = 0;
        let mut progress = |ev: ProgressEvent| {
            if let ProgressEvent::FileStarted { .. } = ev {
                n += 1;
                if n >= 2 {
                    cancel2.cancel();
                }
            }
        };
        let result = scrub_workspace(
            dir.path(),
            0,
            RulePack::BuiltinV1,
            &WorkspaceLimits::for_tests(),
            &cancel,
            Some(&mut progress),
        )
        .unwrap();
        assert!(result.cancelled);
    }

    #[test]
    fn sensitive_filename_forces_review_required() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("alice@example.com.log"), "nothing sensitive here\n").unwrap();
        let cancel = CancelFlag::new();
        let result = scrub_workspace(
            dir.path(),
            0,
            RulePack::BuiltinV1,
            &WorkspaceLimits::for_tests(),
            &cancel,
            None,
        )
        .unwrap();
        assert_eq!(result.safety_status, SafetyStatus::ReviewRequired);
        let f = result
            .files
            .iter()
            .find(|f| f.outcome.path.contains("alice@example.com"))
            .unwrap();
        assert!(f.outcome.reason.as_ref().unwrap().contains("file path"));
    }

    #[test]
    fn unremarkable_filename_stays_clean() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("notes.log"), "nothing sensitive here\n").unwrap();
        let cancel = CancelFlag::new();
        let result = scrub_workspace(
            dir.path(),
            0,
            RulePack::BuiltinV1,
            &WorkspaceLimits::for_tests(),
            &cancel,
            None,
        )
        .unwrap();
        assert_eq!(result.safety_status, SafetyStatus::SafeCopyReady);
    }

    #[test]
    fn max_file_size_excludes() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("big.log"), vec![b'a'; 1000]).unwrap();
        let mut limits = WorkspaceLimits::for_tests();
        limits.max_file_size = 100;
        let cancel = CancelFlag::new();
        let result =
            scrub_workspace(dir.path(), 0, RulePack::BuiltinV1, &limits, &cancel, None).unwrap();
        assert_eq!(result.files[0].outcome.inclusion, FileInclusion::Excluded);
        assert_eq!(result.safety_status, SafetyStatus::ReviewRequired);
    }
}
