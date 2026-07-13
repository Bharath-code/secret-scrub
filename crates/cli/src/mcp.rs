//! Stdio MCP server: one local `scrub` tool for AI agents.
//!
//! Transport: newline-delimited JSON-RPC 2.0 over stdin/stdout (MCP stdio).
//! No network, no filesystem access — the agent passes text content only.
//! Each tool call uses a fresh correlation scope (own placeholder allocator).

use secretscrub_core::{
    scrub, ContentFormat, SafetyStatus, ScrubConfig, ScrubError, StructureStatus, WorkspaceLimits,
    PRODUCT_VERSION,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

/// MCP protocol version negotiated with clients (Claude Code, etc.).
const PROTOCOL_VERSION: &str = "2024-11-05";

const TOOL_NAME: &str = "scrub";

const TOOL_DESCRIPTION: &str = "\
Redact common secrets and identifiers from text on this device (local only, no upload). \
Returns a safe copy plus findings (types, placeholders, and counts only — never secret values). \
Each call is an independent correlation scope: placeholders do not carry across calls. \
Detection covers common patterns only and cannot guarantee every sensitive value is found; \
review the safe copy before sharing. \
Input must be valid UTF-8 text within the same size and line limits as stdin scrubs \
(default max 10 MiB total, 1 MiB per line). Over-limit input returns an error, not a partial safe result.";

/// Run the MCP server until stdin EOF. Returns Ok when the session ends cleanly.
pub fn run_mcp_server() -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let lines = stdin.lock().lines();

    for line in lines {
        let line = line.map_err(|e| format!("read stdin: {e}"))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let msg: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                write_message(
                    &mut stdout,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32700,
                            "message": format!("parse error: {e}")
                        }
                    }),
                )?;
                continue;
            }
        };

        // Notifications have no id (or null) and must not receive a response.
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(Value::Null);

        if id.is_none() || id.as_ref().is_some_and(|v| v.is_null()) {
            // Notification — handle side effects only.
            let _ = (method, params);
            continue;
        }

        let id = id.unwrap();
        let response = match method {
            "initialize" => ok_result(id, initialize_result()),
            "ping" => ok_result(id, json!({})),
            "tools/list" => ok_result(id, tools_list_result()),
            "tools/call" => match handle_tools_call(&params) {
                Ok(result) => ok_result(id, result),
                Err(e) => error_result(id, -32602, e),
            },
            "resources/list" => ok_result(id, json!({ "resources": [] })),
            "prompts/list" => ok_result(id, json!({ "prompts": [] })),
            "" => error_result(id, -32600, "invalid request: missing method".into()),
            other => error_result(id, -32601, format!("method not found: {other}")),
        };
        write_message(&mut stdout, &response)?;
    }
    Ok(())
}

fn write_message(out: &mut impl Write, msg: &Value) -> Result<(), String> {
    let line = serde_json::to_string(msg).map_err(|e| e.to_string())?;
    writeln!(out, "{line}").map_err(|e| format!("write stdout: {e}"))?;
    out.flush().map_err(|e| format!("flush stdout: {e}"))?;
    Ok(())
}

fn ok_result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_result(id: Value, code: i32, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "secretscrub",
            "version": PRODUCT_VERSION
        },
        "instructions": "Use the scrub tool to redact common secrets from text before sharing with models or vendors. Local only; review the safe copy."
    })
}

fn tools_list_result() -> Value {
    json!({
        "tools": [{
            "name": TOOL_NAME,
            "description": TOOL_DESCRIPTION,
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text content to scrub. Pass file contents yourself — this tool never reads paths from disk."
                    },
                    "format": {
                        "type": "string",
                        "enum": ["plain", "json", "yaml", "env"],
                        "description": "Optional structure hint. Defaults to plain text. JSON/YAML/env preserve structure where possible."
                    }
                },
                "required": ["text"],
                "additionalProperties": false
            }
        }]
    })
}

fn handle_tools_call(params: &Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| "tools/call requires params.name".to_string())?;
    if name != TOOL_NAME {
        return Err(format!("unknown tool: {name}"));
    }

    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if !args.is_object() {
        return Err("tools/call arguments must be an object".into());
    }

    match scrub_tool(&args) {
        Ok(payload) => {
            let text = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }],
                "structuredContent": payload,
                "isError": false
            }))
        }
        Err(tool_err) => {
            let text = serde_json::to_string(&tool_err).map_err(|e| e.to_string())?;
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }],
                "structuredContent": tool_err,
                "isError": true
            }))
        }
    }
}

/// Successful scrub payload returned to the agent (never includes secret values).
#[derive(Debug, Serialize)]
struct ScrubToolResult {
    safe_text: String,
    findings: Vec<FindingOut>,
    safety_status: SafetyStatus,
    structure_status: StructureStatus,
    rule_pack_version: String,
    /// Present when structure is unsupported or other non-fatal notes apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

#[derive(Debug, Serialize)]
struct FindingOut {
    detector_type: String,
    placeholder: String,
    occurrences: usize,
}

/// Typed tool error (isError: true). Distinct kinds for agents/tests.
#[derive(Debug, Serialize)]
struct ScrubToolError {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
}

fn scrub_tool(args: &Value) -> Result<ScrubToolResult, ScrubToolError> {
    let text = args
        .get("text")
        .and_then(|t| t.as_str())
        .ok_or_else(|| ScrubToolError {
            error: "invalid_arguments".into(),
            message: "missing required argument: text".into(),
            kind: Some("invalid_arguments".into()),
        })?;

    let format = match args.get("format") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(parse_format(s)?),
        Some(_) => {
            return Err(ScrubToolError {
                error: "invalid_arguments".into(),
                message: "format must be a string: plain | json | yaml | env".into(),
                kind: Some("invalid_arguments".into()),
            });
        }
    };

    // Same bounds as stdin scrubs (WorkspaceLimits defaults).
    let limits = WorkspaceLimits::default();
    let byte_len = text.len() as u64;
    if byte_len > limits.max_file_size {
        return Err(ScrubToolError {
            error: "limit_exceeded".into(),
            message: format!(
                "text exceeds max_file_size {} (got {byte_len} bytes)",
                limits.max_file_size
            ),
            kind: Some("max_file_size".into()),
        });
    }
    for line in text.lines() {
        if line.len() > limits.max_line_length {
            return Err(ScrubToolError {
                error: "limit_exceeded".into(),
                message: format!(
                    "line exceeds max_line_length {} (got {} chars)",
                    limits.max_line_length,
                    line.len()
                ),
                kind: Some("max_line_length".into()),
            });
        }
    }

    // JSON strings are Unicode; reject empty the same as CLI stdin.
    if text.is_empty() {
        return Err(ScrubToolError {
            error: "empty_input".into(),
            message: "input is empty".into(),
            kind: Some("empty_input".into()),
        });
    }

    // Validate no NUL / binary-like content slipped through as a JSON string.
    if text.bytes().any(|b| b == 0) {
        return Err(ScrubToolError {
            error: "invalid_input".into(),
            message: "text contains NUL bytes (binary content is unsupported)".into(),
            kind: Some("invalid_utf8_or_binary".into()),
        });
    }

    let config = ScrubConfig {
        format,
        ..ScrubConfig::default()
    };

    let result = scrub(text, &config).map_err(|e| match e {
        ScrubError::EmptyInput => ScrubToolError {
            error: "empty_input".into(),
            message: "input is empty".into(),
            kind: Some("empty_input".into()),
        },
    })?;

    Ok(ScrubToolResult {
        safe_text: result.text,
        findings: result
            .findings
            .into_iter()
            .map(|f| FindingOut {
                detector_type: f.detector_type,
                placeholder: f.placeholder,
                occurrences: f.occurrences,
            })
            .collect(),
        safety_status: result.safety_status,
        structure_status: result.structure_status,
        rule_pack_version: result.rule_pack_version,
        note: result.note,
    })
}

fn parse_format(s: &str) -> Result<ContentFormat, ScrubToolError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "plain" | "text" | "plain_text" | "plaintext" => Ok(ContentFormat::PlainText),
        "json" => Ok(ContentFormat::Json),
        "yaml" | "yml" => Ok(ContentFormat::Yaml),
        "env" => Ok(ContentFormat::Env),
        other => Err(ScrubToolError {
            error: "invalid_arguments".into(),
            message: format!(
                "unknown format '{other}'; expected plain | json | yaml | env"
            ),
            kind: Some("invalid_format".into()),
        }),
    }
}
