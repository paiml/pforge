//! End-to-end verification of the MCP stdio transport, against the SHIPPED
//! BINARY.
//!
//! `pforge serve` is advertised in `--help`, so under the dogfood protocol's
//! transport-absence rule it must be DECLARED and exercised: an undeclared
//! transport is an unverified one.
//!
//! THE INVARIANT: every name returned by `tools/list` must be callable via
//! `tools/call`. An MCP client — usually an LLM — reads `tools/list` and
//! believes it, so an advertised-but-uncallable tool surfaces at use time as a
//! confusing protocol error rather than as a missing capability.
//!
//! Both sides of that invariant are tested here:
//!   · a `cli` tool IS registered, so the server must start and dispatch it;
//!   · a `native` tool is NOT registered by the generic binary, so the server
//!     must REFUSE TO START rather than advertise it.
//!
//! Readiness is read FROM THE CHILD (its reply to `initialize`), never from a
//! sleep. A sleep-based test is a race that passes on a fast machine.
//!
//! Declared in Cargo.toml as `[package.metadata.transports] mcp`.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_pforge")
}

/// A unique scratch dir. Deliberately not shared `/tmp` state: two concurrent
/// cargo test threads writing the same path would race, which is a defect this
/// fleet has already paid for once.
fn scratch(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "pforge-e2e-{tag}-{}-{}",
        std::process::id(),
        format!("{:?}", std::thread::current().id())
            .replace(|c: char| !c.is_ascii_alphanumeric(), "")
    ));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).expect("scratch dir");
    p
}

/// A project whose single tool IS dispatchable by the generic binary.
fn project_with_cli_tool(tag: &str) -> PathBuf {
    let dir = scratch(tag);
    std::fs::write(
        dir.join("pforge.yaml"),
        r#"forge:
  name: cli-demo
  version: 0.1.0
  transport: stdio

tools:
  - type: cli
    name: echo_it
    description: "Echo a fixed string"
    command: echo
    args: ["dogfood-ok"]
"#,
    )
    .expect("write pforge.yaml");
    dir
}

/// Speak a sequence of JSON-RPC messages; return (reply lines, exit status).
/// Only messages carrying an `id` are waited on — blocking for a reply to a
/// notification would hang forever.
fn converse(proj: &PathBuf, messages: &[&str]) -> (Vec<String>, std::process::ExitStatus) {
    let mut child = Command::new(bin())
        .arg("serve")
        .current_dir(proj)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("`pforge serve` must spawn");

    let mut replies = Vec::new();
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        let mut reader = BufReader::new(stdout);

        for msg in messages {
            writeln!(stdin, "{msg}").expect("write");
            stdin.flush().expect("flush");
            if msg.contains("\"id\"") {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                replies.push(line);
            }
        }
        child.stdout = Some(reader.into_inner());
    }
    drop(child.stdin.take());
    let status = child.wait().expect("child must terminate");
    (replies, status)
}

const INIT: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"dogfood","version":"1"}}}"#;
const INITIALIZED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;

#[test]
fn initialize_returns_a_well_formed_mcp_result() {
    let proj = project_with_cli_tool("init");
    let (replies, status) = converse(&proj, &[INIT]);

    let line = replies.first().map(String::as_str).unwrap_or("");
    assert!(
        !line.trim().is_empty(),
        "server produced no response — an MCP transport that answers nothing is \
         indistinguishable from one that is not wired up"
    );
    for needle in [
        r#""jsonrpc":"2.0""#,
        r#""id":1"#,
        r#""result""#,
        "protocolVersion",
    ] {
        assert!(
            line.contains(needle),
            "initialize missing {needle}; got: {line}"
        );
    }
    assert!(
        line.contains(r#""name":"cli-demo""#),
        "serverInfo must come from the config actually loaded, not a constant; got: {line}"
    );
    assert!(
        status.success(),
        "server exited {:?} after a clean session",
        status.code()
    );
}

/// The invariant, on the side that must WORK.
#[test]
fn every_advertised_tool_is_callable() {
    let proj = project_with_cli_tool("dispatch");
    let (replies, _) = converse(
        &proj,
        &[
            INIT,
            INITIALIZED,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo_it","arguments":{}}}"#,
        ],
    );

    let list = replies.get(1).cloned().unwrap_or_default();
    assert!(
        list.contains("echo_it"),
        "tools/list must advertise the configured tool; got: {list}"
    );

    let call = replies.get(2).cloned().unwrap_or_default();
    assert!(
        !call.contains("not found") && !call.contains("Tool not found"),
        "tools/list advertised `echo_it` but tools/call could not dispatch it — \
         list and call must agree (paiml/pforge#12); got: {call}"
    );
}

/// The invariant, on the side that must REFUSE.
///
/// `pforge new` scaffolds a `type: native` tool whose handler lives in the
/// generated project's own src/. The generic `pforge serve` has no knowledge of
/// that Rust, so it cannot dispatch it. Before the guard this shipped as a
/// server that advertised `hello` and answered "Tool not found" on call.
#[test]
fn native_scaffold_refuses_to_start_rather_than_advertise_what_it_cannot_dispatch() {
    let root = scratch("native");
    let out = Command::new(bin())
        .args(["new", "demo"])
        .current_dir(&root)
        .output()
        .expect("`pforge new` must run");
    assert!(out.status.success(), "`pforge new demo` failed");
    let proj = root.join("demo");
    assert!(
        proj.join("pforge.yaml").is_file(),
        "scaffold produced no pforge.yaml"
    );

    let child = Command::new(bin())
        .arg("serve")
        .current_dir(&proj)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    let out = child.wait_with_output().expect("wait");
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        !out.status.success(),
        "serve must NOT start when a declared tool has no registered handler — \
         starting means advertising a tool that always fails on call"
    );
    assert!(
        stderr.contains("refusing to start") && stderr.contains("hello"),
        "the refusal must name the offending tool so the operator can act on it; \
         stderr was:\n{stderr}"
    );
    assert!(
        stderr.contains("pforge build"),
        "the refusal must say how to fix it (build the project's own binary); \
         stderr was:\n{stderr}"
    );
}

#[test]
fn malformed_request_does_not_crash_the_server() {
    let proj = project_with_cli_tool("bad");
    let (_replies, status) = converse(&proj, &[r#"{ this is not json "#]);
    assert!(
        status.code().is_some(),
        "server did not exit normally on malformed input (signal: {status:?})"
    );
}
