//! Black-box MCP stdio tests: initialize → list tools → call scrub.

use assert_cmd::cargo::cargo_bin;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

const AWS: &str = "AKIAIOSFODNN7EXAMPLE";

struct McpSession {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
    next_id: i64,
}

impl McpSession {
    fn start() -> Self {
        let bin = cargo_bin!("secretscrub");
        let mut child = Command::new(bin)
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn secretscrub mcp");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        Self {
            child,
            stdin,
            stdout,
            next_id: 1,
        }
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        writeln!(self.stdin, "{}", serde_json::to_string(&msg).unwrap()).expect("write");
        self.stdin.flush().expect("flush");
        self.read_response(id)
    }

    fn notify(&mut self, method: &str, params: Value) {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        writeln!(self.stdin, "{}", serde_json::to_string(&msg).unwrap()).expect("write");
        self.stdin.flush().expect("flush");
    }

    fn read_response(&mut self, expected_id: i64) -> Value {
        let mut line = String::new();
        self.stdout.read_line(&mut line).expect("read response line");
        assert!(
            !line.trim().is_empty(),
            "expected JSON-RPC response, got empty line (process may have exited)"
        );
        let v: Value = serde_json::from_str(line.trim()).expect("parse response JSON");
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["id"], expected_id);
        v
    }

    fn shutdown(mut self) {
        drop(self.stdin);
        let _ = self.child.wait();
    }
}

fn handshake(session: &mut McpSession) {
    let init = session.request(
        "initialize",
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "secretscrub-test", "version": "0.0.0" }
        }),
    );
    assert!(init.get("error").is_none(), "initialize error: {init}");
    let result = &init["result"];
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert_eq!(result["serverInfo"]["name"], "secretscrub");
    assert!(result["capabilities"]["tools"].is_object());

    session.notify("notifications/initialized", json!({}));
}

#[test]
fn mcp_initialize_and_list_scrub_tool() {
    let mut s = McpSession::start();
    handshake(&mut s);

    let listed = s.request("tools/list", json!({}));
    assert!(listed.get("error").is_none(), "tools/list error: {listed}");
    let tools = listed["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "scrub");
    let desc = tools[0]["description"].as_str().unwrap_or("");
    assert!(
        desc.contains("common patterns") || desc.contains("cannot guarantee"),
        "tool description must state detection limits: {desc}"
    );
    assert!(desc.to_ascii_lowercase().contains("local") || desc.contains("device"));

    s.shutdown();
}

#[test]
fn mcp_scrub_redacts_secret_and_never_leaks_in_envelope() {
    let mut s = McpSession::start();
    handshake(&mut s);

    let call = s.request(
        "tools/call",
        json!({
            "name": "scrub",
            "arguments": {
                "text": format!("before {AWS} after")
            }
        }),
    );
    assert!(call.get("error").is_none(), "tools/call error: {call}");
    let result = &call["result"];
    assert_eq!(result["isError"], false);

    // Full envelope must never contain the raw secret.
    let envelope = serde_json::to_string(&call).unwrap();
    assert!(
        !envelope.contains(AWS),
        "MCP response envelope must not contain secret values"
    );

    let body: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    let safe = body["safe_text"].as_str().unwrap();
    assert!(!safe.contains(AWS));
    assert!(safe.contains("before"));
    assert!(safe.contains("[AWS_ACCESS_KEY#"));
    assert!(!body["findings"].as_array().unwrap().is_empty());
    assert!(body["rule_pack_version"].as_str().is_some());
    assert!(body["safety_status"].is_string());
    assert!(body["structure_status"].is_string());

    // structuredContent mirrors payload when present
    if let Some(sc) = result.get("structuredContent") {
        assert!(!serde_json::to_string(sc).unwrap().contains(AWS));
    }

    s.shutdown();
}

#[test]
fn mcp_scrub_over_limit_returns_typed_error_not_safe() {
    let mut s = McpSession::start();
    handshake(&mut s);

    // Exceed max_line_length (1 MiB default) with a single huge line.
    let huge_line = "A".repeat(1024 * 1024 + 10);
    let call = s.request(
        "tools/call",
        json!({
            "name": "scrub",
            "arguments": { "text": huge_line }
        }),
    );
    assert!(call.get("error").is_none(), "protocol error: {call}");
    let result = &call["result"];
    assert_eq!(result["isError"], true, "expected tool-level error: {result}");

    let body: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(body["error"], "limit_exceeded");
    assert_eq!(body["kind"], "max_line_length");
    assert!(body.get("safe_text").is_none());

    s.shutdown();
}

#[test]
fn mcp_scrub_empty_input_typed_error() {
    let mut s = McpSession::start();
    handshake(&mut s);

    let call = s.request(
        "tools/call",
        json!({
            "name": "scrub",
            "arguments": { "text": "" }
        }),
    );
    let result = &call["result"];
    assert_eq!(result["isError"], true);
    let body: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(body["error"], "empty_input");

    s.shutdown();
}

#[test]
fn mcp_scrub_null_bytes_typed_error() {
    let mut s = McpSession::start();
    handshake(&mut s);

    // JSON allows \u0000 in strings.
    let call = s.request(
        "tools/call",
        json!({
            "name": "scrub",
            "arguments": { "text": "ok\u{0000}bad" }
        }),
    );
    let result = &call["result"];
    assert_eq!(result["isError"], true);
    let body: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(body["error"], "invalid_input");
    assert_eq!(body["kind"], "invalid_utf8_or_binary");

    s.shutdown();
}

#[test]
fn mcp_scrub_json_format_hint() {
    let mut s = McpSession::start();
    handshake(&mut s);

    let payload = format!(r#"{{"key":"{AWS}","n":1}}"#);
    let call = s.request(
        "tools/call",
        json!({
            "name": "scrub",
            "arguments": {
                "text": payload,
                "format": "json"
            }
        }),
    );
    let result = &call["result"];
    assert_eq!(result["isError"], false);
    let envelope = serde_json::to_string(&call).unwrap();
    assert!(!envelope.contains(AWS));

    let body: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    let safe = body["safe_text"].as_str().unwrap();
    assert!(!safe.contains(AWS));
    // Structure-preserving: still valid-looking JSON with key name.
    assert!(safe.contains("\"key\"") || safe.contains("key"));

    s.shutdown();
}

#[test]
fn mcp_unknown_tool_is_jsonrpc_error() {
    let mut s = McpSession::start();
    handshake(&mut s);

    let call = s.request(
        "tools/call",
        json!({
            "name": "not_a_tool",
            "arguments": {}
        }),
    );
    assert!(call.get("error").is_some());
    assert!(call["error"]["message"]
        .as_str()
        .unwrap_or("")
        .contains("unknown tool"));

    s.shutdown();
}
