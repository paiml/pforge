//! Does pforge actually work as an MCP server?
//!
//! Every other test in this workspace answers a question one layer below that:
//! does the YAML parse, does the registry dispatch, does a handler return what
//! it was given. Those are worth having, but none of them speaks the protocol.
//! `e2e_test.rs::test_stdio_transport_config`, for instance, deserializes a
//! config and asserts `config.forge.transport == TransportType::Stdio` — it
//! never constructs a transport.
//!
//! So when `pmcp` went 1.8 -> 2.18 (ten releases, a major version, and the
//! crate that implements the entire wire protocol), all 228 tests passed and
//! not one of them had exchanged a JSON-RPC frame with a pforge server. "It
//! compiles" was standing in for "it works".
//!
//! These tests spawn the real `pforge` binary, speak MCP over its stdin and
//! stdout, and assert on the bytes that come back.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

/// A config with one `cli` tool. `cli` is used rather than `native` on purpose:
/// a native tool needs a compiled handler in the server binary, which would put
/// the fixture back inside the crate under test.
const FIXTURE: &str = r#"
forge:
  name: protocol-fixture
  version: 0.1.0
  transport: stdio

tools:
  - type: cli
    name: echo_tool
    description: "Echo a fixed string, so the response is checkable"
    command: echo
    args: ["pforge-protocol-ok"]
"#;

fn write_fixture(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("pforge.yaml");
    std::fs::write(&path, FIXTURE).expect("write fixture config");
    path
}

/// Path to the `pforge` binary built by this workspace.
///
/// `CARGO_BIN_EXE_` is not available here — that env var is only set for the
/// crate that declares the binary — so it is located relative to this test
/// executable, which cargo places in the same target profile directory.
fn pforge_bin() -> std::path::PathBuf {
    let mut p = std::env::current_exe().expect("test exe path");
    p.pop(); // deps/
    if p.ends_with("deps") {
        p.pop();
    }
    p.join("pforge")
}

struct Session {
    child: std::process::Child,
}

impl Session {
    fn start(config: &std::path::Path) -> Option<Self> {
        let bin = pforge_bin();
        if !bin.exists() {
            // The binary is built by `cargo test --workspace`, but a filtered
            // run of just this crate may not have produced it. Say so rather
            // than pass silently — a test that cannot run is not a test that
            // passed.
            eprintln!(
                "SKIP: {} not built; run `cargo build -p pforge-cli` first",
                bin.display()
            );
            return None;
        }
        let child = Command::new(bin)
            .arg("serve")
            .arg("--config")
            .arg(config)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn pforge serve");
        Some(Self { child })
    }

    /// Write one JSON-RPC frame, then read frames until one carries `id`.
    ///
    /// Non-JSON lines are collected and reported rather than skipped: on a
    /// stdio transport, stdout belongs to the protocol, and anything else on it
    /// is a bug in the server, not noise to filter out.
    fn request(
        &mut self,
        frame: &str,
        id: u64,
    ) -> Result<(serde_json::Value, Vec<String>), String> {
        let stdin = self.child.stdin.as_mut().expect("stdin piped");
        writeln!(stdin, "{frame}").map_err(|e| format!("write: {e}"))?;
        stdin.flush().map_err(|e| format!("flush: {e}"))?;

        let stdout = self.child.stdout.as_mut().expect("stdout piped");
        let mut reader = BufReader::new(stdout);
        let mut non_protocol: Vec<String> = Vec::new();

        for _ in 0..64 {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => return Err(format!("read: {e}")),
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match serde_json::from_str::<serde_json::Value>(trimmed) {
                Ok(v) => {
                    if v.get("id").and_then(serde_json::Value::as_u64) == Some(id) {
                        return Ok((v, non_protocol));
                    }
                }
                Err(_) => non_protocol.push(trimmed.to_string()),
            }
        }
        Err(format!(
            "no response with id={id}. Non-JSON lines on stdout ({}): {:?}",
            non_protocol.len(),
            non_protocol
        ))
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn init_frame() -> String {
    serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "pforge-protocol-test", "version": "1"}
        }
    })
    .to_string()
}

/// stdout on a stdio transport carries the protocol and nothing else.
///
/// `commands/serve.rs` opens with four `println!` calls — "Starting pforge
/// server...", the config path, the server name, the transport — and `println!`
/// writes to stdout, which is the same channel the JSON-RPC frames use. Any
/// client that parses stdout line-by-line sees those first.
///
/// This is the test that fails on that, and it fails with the offending lines
/// quoted rather than with a timeout, because "the server never answered" and
/// "the server answered after four lines of prose" are different bugs.
#[test]
fn stdout_carries_only_jsonrpc() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = write_fixture(dir.path());
    let Some(mut session) = Session::start(&cfg) else {
        return;
    };
    std::thread::sleep(Duration::from_millis(300));

    match session.request(&init_frame(), 1) {
        Ok((v, non_protocol)) => {
            assert_eq!(v["jsonrpc"], "2.0");
            assert!(
                v.get("result").is_some(),
                "initialize returned an error: {v}"
            );
            // The assertion this test is named for. Reaching a valid response is
            // NOT the same claim: a client that reads stdout line-by-line hits
            // whatever came first. An earlier version of this test only
            // reported these lines on failure, so it passed while stdout was
            // polluted — the name promised more than the body checked.
            assert!(
                non_protocol.is_empty(),
                "stdout carries the MCP protocol on a stdio transport, but {} \
                 non-JSON line(s) were written to it before the response: {:?}",
                non_protocol.len(),
                non_protocol
            );
        }
        Err(e) => panic!("initialize failed on the real wire protocol: {e}"),
    }
}

/// The tool declared in YAML is the tool the server advertises.
///
/// This is the assertion the config-parsing tests cannot make: they prove the
/// YAML deserialized, not that the tool survived registration, adapter
/// construction and pmcp's own schema handling to appear in `tools/list`.
#[test]
fn declared_tool_is_advertised_over_the_protocol() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cfg = write_fixture(dir.path());
    let Some(mut session) = Session::start(&cfg) else {
        return;
    };
    std::thread::sleep(Duration::from_millis(300));

    session
        .request(&init_frame(), 1)
        .expect("initialize must succeed before tools/list");

    let (listed, _) = session
        .request(
            &serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}})
                .to_string(),
            2,
        )
        .expect("tools/list must return a response");

    let tools = listed["result"]["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("tools/list has no tools array: {listed}"));
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(
        names.contains(&"echo_tool"),
        "the tool declared in pforge.yaml is not advertised by the server. got {names:?}"
    );
}
