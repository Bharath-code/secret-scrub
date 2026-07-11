# secretscrub-core

Local-first redaction engine for SecretScrub. No network I/O.

## API

```rust
use secretscrub_core::{scrub, ScrubConfig, atomic_write, SafetySummary};

let result = scrub(content, &ScrubConfig::default())?;
// result.text, result.findings, result.safety_status, result.rule_pack_version
```

Placeholders are correlated within one scrub session only.
