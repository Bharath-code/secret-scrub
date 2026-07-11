//! Content format classification for structure-preserving transforms.

use std::path::Path;

/// Supported / unsupported content formats for private beta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentFormat {
    PlainText,
    Json,
    Yaml,
    Env,
    /// Documented deferred for private beta (issue 04).
    TomlUnsupported,
    /// Binary or unknown non-text.
    BinaryUnsupported,
}

impl ContentFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            ContentFormat::PlainText => "plain_text",
            ContentFormat::Json => "json",
            ContentFormat::Yaml => "yaml",
            ContentFormat::Env => "env",
            ContentFormat::TomlUnsupported => "toml",
            ContentFormat::BinaryUnsupported => "binary",
        }
    }

    pub fn is_text_supported(self) -> bool {
        matches!(
            self,
            ContentFormat::PlainText
                | ContentFormat::Json
                | ContentFormat::Yaml
                | ContentFormat::Env
        )
    }
}

/// Infer format from file path extension (case-insensitive).
pub fn format_from_path(path: &Path) -> ContentFormat {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if name == ".env" || name.starts_with(".env.") || name.ends_with(".env") {
        return ContentFormat::Env;
    }

    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("json") => ContentFormat::Json,
        Some("yaml") | Some("yml") => ContentFormat::Yaml,
        Some("toml") => ContentFormat::TomlUnsupported,
        Some("log") | Some("txt") | Some("md") | Some("rs") | Some("js") | Some("ts")
        | Some("py") | Some("go") | Some("java") | Some("sh") | Some("bash") => {
            ContentFormat::PlainText
        }
        Some("bin") | Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("pdf")
        | Some("zip") | Some("gz") | Some("tar") | Some("exe") | Some("dll") | Some("so")
        | Some("dylib") => ContentFormat::BinaryUnsupported,
        _ => ContentFormat::PlainText,
    }
}

/// Sniff binary content: high NUL ratio or invalid UTF-8 already handled by reader.
pub fn looks_binary(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let sample = &bytes[..bytes.len().min(8192)];
    let nuls = sample.iter().filter(|b| **b == 0).count();
    nuls > 0 || sample.iter().filter(|b| **b < 9 && **b != b'\n' && **b != b'\r' && **b != b'\t').count() > sample.len() / 8
}
