# SecretScrub

**Share production context safely in seconds.**

SecretScrub is a **local-first** desktop app and CLI that turns logs, configs, and incident bundles into **reviewed safe copies**. It detects common secrets and identifiers, replaces them with consistent semantic placeholders, preserves useful structure, and exports a shareable artifact — **without uploading the original**.

> Status: early product definition. Core engine and public releases are not shipped yet.

## Why it exists

Debugging now means pasting logs into AI tools or sending bundles to vendors. Manual redaction is slow and error-prone; over-stripping kills useful context. Enterprise secret scanners find leaks in repos and pipelines — they are not built for “make **this** file safe to share **now**.”

## Product promise

| We do | We do not |
| --- | --- |
| Scan and transform **on your device** | Claim every sensitive value will be found |
| Side-by-side **review** before export | Validate whether a credential is still active |
| Keep **originals immutable** | Host or store your artifacts in our cloud |
| Preserve structure for supported formats | Replace your security program or DLP |

Full honesty policy: [`.scratch/secretscrub/TRUST.md`](.scratch/secretscrub/TRUST.md)

## Planned surfaces

- **macOS desktop** — drop → scan → review → export  
- **CLI** — path/stdin, machine-readable findings, automation-friendly exit codes  
- **MCP server** — `secretscrub mcp` exposes a local `scrub` tool for AI agents  
- **Free CLI / Pro desktop** — see business plan for packaging intent  

## Repository layout

| Path | Purpose |
| --- | --- |
| [`CONTEXT.md`](CONTEXT.md) | Domain vocabulary |
| [`AGENTS.md`](AGENTS.md) | Agent workflow conventions |
| [`.scratch/secretscrub/PRD.md`](.scratch/secretscrub/PRD.md) | Product requirements |
| [`.scratch/secretscrub/BUSINESS-PLAN.md`](.scratch/secretscrub/BUSINESS-PLAN.md) | Market and GTM |
| [`.scratch/secretscrub/ODDS-IMPROVEMENT.md`](.scratch/secretscrub/ODDS-IMPROVEMENT.md) | Weakness/threat counterplay |
| [`.scratch/secretscrub/VALIDATION.md`](.scratch/secretscrub/VALIDATION.md) | Interview script and log |
| [`.scratch/secretscrub/COMPETITIVE.md`](.scratch/secretscrub/COMPETITIVE.md) | Battle card |
| [`.scratch/secretscrub/TRUST.md`](.scratch/secretscrub/TRUST.md) | Detection honesty |

Application code will live in a Rust workspace (core engine + CLI + Tauri desktop) once validation gates pass.

## Privacy (default)

- No account required for the core scrub workflow (v1 intent).  
- No telemetry by default.  
- Optional diagnostics never include artifact content, secret values, or secret fingerprints.  
- Placeholders correlate **within one workspace only**, not across separate exports.

## CLI (local only)

Requires [Rust](https://rustup.rs/) 1.75+.

```bash
# Build
cargo build -p secretscrub-cli

# Scrub stdin → stdout (nothing is uploaded)
echo 'aws_access_key_id=AKIAIOSFODNN7EXAMPLE' | cargo run -q -p secretscrub-cli -- scrub

# Scrub a file → atomic safe copy + summary (source is never modified)
cargo run -q -p secretscrub-cli -- scrub ./app.log -o ./app.safe.log --summary ./app.summary.json

# Structure-preserving JSON / YAML / env (by extension)
cargo run -q -p secretscrub-cli -- scrub ./config.json -o ./config.safe.json

# Folder workspace → correlated safe tree
cargo run -q -p secretscrub-cli -- scrub ./incident-bundle -o ./incident-bundle.safe --format json

# Machine-readable findings only (no secret values)
cargo run -q -p secretscrub-cli -- scrub ./app.log --format json

# Detection-only for CI / pre-commit (no files written; exit 2 if findings)
cargo run -q -p secretscrub-cli -- scrub --check ./app.log
echo 'AKIAIOSFODNN7EXAMPLE' | cargo run -q -p secretscrub-cli -- scrub --check

# Tests
cargo test
```

Binary name: `secretscrub` (package `secretscrub-cli`).

### Exit codes (automation contract)

| Code | Meaning |
| --- | --- |
| **0** | Clean (`safe_copy_ready`) |
| **1** | Failure (IO, empty input, limits, export error) |
| **2** | Completed with **review required** (or secrets found in `--check` mode) |
| **3** | **Unsupported** input (nothing safe produced, e.g. TOML/binary-only) |

### Pre-commit and CI

Install the binary (`cargo install --path crates/cli`), then in another repo:

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/Bharath-code/secret-scrub
    rev: main   # pin to a tag/sha in production
    hooks:
      - id: secretscrub
```

Hook definition lives in [`.pre-commit-hooks.yaml`](.pre-commit-hooks.yaml) (`secretscrub scrub --check` on text files).

GitHub Actions example:

```yaml
- name: Install SecretScrub
  run: cargo install --path crates/cli --locked
- name: Check for secrets
  run: secretscrub scrub --check $(git ls-files)
```

`--check` never creates or modifies files. Stderr reports detector types and counts only (never secret values).

### MCP server (AI agents)

`secretscrub mcp` speaks **MCP over stdio** (newline-delimited JSON-RPC). It exposes one tool, **`scrub`**, that redacts text in-process — **no network, no filesystem paths**. The agent passes content; each call is its own placeholder correlation scope.

Register with [Claude Code](https://docs.anthropic.com/en/docs/claude-code):

```bash
# After installing the binary onto PATH
claude mcp add secretscrub -- secretscrub mcp

# Or from a cargo build without installing
claude mcp add secretscrub -- cargo run -q -p secretscrub-cli -- mcp
```

Tool contract:

| Field | Direction | Notes |
| --- | --- | --- |
| `text` | input (required) | UTF-8 text to scrub |
| `format` | input (optional) | `plain` \| `json` \| `yaml` \| `env` |
| `safe_text` | output | Redacted copy |
| `findings` | output | Types, placeholders, counts only (never secret values) |
| `safety_status` / `structure_status` / `rule_pack_version` | output | Same honesty model as CLI |

Over-limit or empty input returns a **typed tool error** (`isError: true`) — never a partial result marked safe. Same size/line limits as stdin scrubs (default 10 MiB / 1 MiB per line).

### Limits (folder / large input)

`--max-depth`, `--max-file-size`, `--max-files`, `--max-line-length` (defaults in engine: depth 8, 10 MiB/file, 500 files, 1 MiB/line). Symlinks are not followed.

Processing is **on-device only**. Detection covers common patterns; it cannot guarantee every sensitive value is found. Review the safe copy before sharing. See [`docs/detector-changelog.md`](docs/detector-changelog.md). **TOML is unsupported** in private beta.

**Verify it works:** step-by-step checks in [`docs/HOW-TO-VERIFY.md`](docs/HOW-TO-VERIFY.md).

**Marketing site:** open [`landing/index.html`](landing/index.html) or `npx serve landing`.

## Development

Workspace layout:

| Crate | Path | Role |
| --- | --- | --- |
| `secretscrub-core` | `crates/core` | Redaction engine, rule pack, export helpers |
| `secretscrub-cli` | `crates/cli` | CLI adapter |

Implementation issues: [`.scratch/secretscrub/ISSUES.md`](.scratch/secretscrub/ISSUES.md)

Near-term: issues **01 → 02 → 03 → 05** (single-file CLI spine), then folders / desktop.

## License

TBD.
