# 06 — Folder workspace and multi-file bundle scrub

Status: needs-triage  
Type: AFK  
Parent: `.scratch/secretscrub/PRD.md` (private beta)

## What to build

Extend the **artifact workspace** so a user can scrub a folder of logs/configs as one workspace: enumerate files with size/recursion limits, share **value correlation** across files in the bundle, export a safe bundle directory, and report per-file support/exclusion reasons.

End-to-end: fixture incident folder with correlated token across two files → same placeholder in both safe copies; one unsupported binary stub excluded with review-required overall status.

## Acceptance criteria

- [ ] CLI accepts a directory path; applies explicit max depth, max file size, max file count (configurable with safe defaults)
- [ ] Does not follow unbounded symlinks; symlink escape attempts fail closed with clear errors
- [ ] Placeholder correlation is workspace-scoped across all included files
- [ ] Export produces a parallel safe tree (or archive policy documented); originals immutable
- [ ] Unsupported/partial files listed with reasons; overall status **Review required** when any file blocks a full safe claim
- [ ] Integration tests: multi-file correlation, limit enforcement, exclusion reporting

## User stories covered

3, 5 (multi-file), 20, 35

## Blocked by

- `02-placeholders-value-correlation`
- `05-atomic-export-safety-summary`
- `04-structured-json-yaml-env` (preferred for mixed bundles)

## Comments

-
