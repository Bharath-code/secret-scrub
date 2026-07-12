# How to check SecretScrub is working

Practical verification for the local-first CLI and core engine.  
All examples use **synthetic** fixture secrets only — never put real credentials in the repo or logs.

## 1. Test suite (strongest check)

```bash
cd /path/to/rust-product   # or your clone path
cargo test
cargo clippy -p secretscrub-core -p secretscrub-cli --all-targets -- -D warnings
```

All tests should pass. Coverage includes detectors, placeholders, JSON/YAML/env structure, folder workspaces, exit codes, and atomic export.

## 2. Manual CLI smoke tests

### Stdin scrub

```bash
echo 'aws_access_key_id=AKIAIOSFODNN7EXAMPLE request=abc' \
  | cargo run -q -p secretscrub-cli -- scrub
```

**Expect:** the key becomes something like `[AWS_ACCESS_KEY#1]`; `request=abc` stays.

### File → safe copy (source unchanged)

```bash
cargo run -q -p secretscrub-cli -- scrub ./fixtures/aws_log.txt \
  -o /tmp/aws.safe.log --summary /tmp/aws.summary.json

# Original still has the fake key
grep AKIA ./fixtures/aws_log.txt

# Safe copy does not
grep AKIA /tmp/aws.safe.log && echo FAIL || echo OK

# Summary has counts, not secrets
cat /tmp/aws.summary.json
```

### Structure-preserving JSON

```bash
cargo run -q -p secretscrub-cli -- scrub ./fixtures/sample.json \
  -o /tmp/out.json

python3 -c "import json; print(json.load(open('/tmp/out.json')))"
```

**Expect:** still valid JSON; fields like `count` / `service` intact; AWS-shaped key redacted.

Also useful:

```bash
cargo run -q -p secretscrub-cli -- scrub ./fixtures/sample.yaml -o /tmp/out.yaml
cargo run -q -p secretscrub-cli -- scrub ./fixtures/sample.env -o /tmp/out.env
```

### Folder workspace (correlated placeholders)

```bash
mkdir -p /tmp/ss-bundle
cp fixtures/aws_log.txt fixtures/repeated_aws.txt /tmp/ss-bundle/

cargo run -q -p secretscrub-cli -- scrub /tmp/ss-bundle \
  -o /tmp/ss-safe --format json

ls /tmp/ss-safe
# Same secret across files should share one placeholder token
```

### Exit codes

```bash
cargo run -q -p secretscrub-cli -- scrub ./fixtures/sample.json --format json >/dev/null
echo $?   # 0 = clean

printf '{bad' > /tmp/x.json
cargo run -q -p secretscrub-cli -- scrub /tmp/x.json --format json >/dev/null
echo $?   # 2 = review required

printf 'k="v"\n' > /tmp/x.toml
cargo run -q -p secretscrub-cli -- scrub /tmp/x.toml --format json >/dev/null
echo $?   # 3 = unsupported
```

| Code | Meaning |
| --- | --- |
| **0** | Clean (`safe_copy_ready`) |
| **1** | Failure (IO, empty input, limits, export error) |
| **2** | Review required |
| **3** | Unsupported (nothing safe produced, e.g. TOML/binary-only) |

### Machine-readable findings only

```bash
cargo run -q -p secretscrub-cli -- scrub ./fixtures/multi_secret.txt --format json
```

**Expect:** JSON on stdout with `findings`, `replacement_counts`, `rule_pack_version`; **no** raw `AKIA…` or other secret material.

### Limits (optional)

```bash
# Should fail with max_file_size (exit 1)
cargo run -q -p secretscrub-cli -- scrub ./fixtures/multi_secret.txt \
  --max-file-size 10
```

## 3. Install binary (optional)

```bash
cargo install --path crates/cli
secretscrub scrub --help
secretscrub scrub ./fixtures/multi_secret.txt -o /tmp/multi.safe.txt
```

## Quick “it’s healthy” checklist

| Check | Pass when |
| --- | --- |
| `cargo test` | All green |
| Stdin / file scrub | Secrets → `[TYPE#N]`, context kept |
| Source file | Unchanged after `-o` |
| JSON / YAML / env fixtures | Still parse / structure preserved after scrub |
| Folder scrub | Safe tree written; shared placeholders |
| Exit codes | 0 / 2 / 3 as above |
| Summary / `--format json` | No raw `AKIA…` (or other fixture secrets) in JSON |

## Related docs

- Product requirements: [`.scratch/secretscrub/PRD.md`](../.scratch/secretscrub/PRD.md)
- Trust / non-claims: [`.scratch/secretscrub/TRUST.md`](../.scratch/secretscrub/TRUST.md)
- Detector changelog: [`detector-changelog.md`](./detector-changelog.md)
- Root overview: [`../README.md`](../README.md)
