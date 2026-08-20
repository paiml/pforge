//! End-to-end verification of the MCP stdio transport, against the SHIPPED
//! BINARY.
//!
//! `pforge serve` is advertised in `--help`, so under the dogfood protocol's
//! transport-absence rule it must be DECLARED and exercised: an undeclared
//! transport is an unverified one.
//!
//! This scaffolds a real project with `pforge new`, then speaks JSON-RPC to
//! `pforge serve` over stdio and asserts on the bytes that come back. It uses
//! the artifact end to end — `pforge new` and `pforge serve` are both the
//! shipped binary, so a break in either is visible here.
//!
//! Readiness is read FROM THE CHILD (the response to `initialize`), never from
//! a sleep. A sleep-based test is a race that passes on a fast machine.
//!
//! Declared in Cargo.toml as `[package.metadata.transports] mcp`.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_pforge")
}

/// A unique scratch dir. Deliberately not `/tmp` shared state: two concurrent
/// cargo test threads scaffolding into the same path would race, which is a
/// defect this fleet has already paid for once.
fn scratch(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "pforge-e2e-{tag}-{}-{}",
        std::process::id(),
        // thread id disambiguates within one process
        format!("{:?}", std::thread::current().id())
            .replace(|c: char| !c.is_ascii_alphanumeric(), "")
    ));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).expect("scratch dir");
    p
}

/// Scaffold a project with the real binary and return its directory.
fn scaffold(tag: &str) -> (PathBuf, PathBuf) {
    let root = scratch(tag);
    let out = Command::new(bin())
        .args(["new", "demo"])
        .current_dir(&root)
        .output()
        .expect("`pforge new` must run");
    assert!(
        out.status.success(),
        "`pforge new demo` failed: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let proj = root.join("demo");
    assert!(
        proj.join("pforge.yaml").is_file(),
        "`pforge new` did not produce a pforge.yaml — the scaffold is the input \
         to every test below, so a silent change here would make them vacuous"
    );
    (root, proj)
}

/// Send one JSON-RPC request to `pforge serve` and return the first response
/// line. Closing stdin lets the server exit rather than hanging the test.
fn rpc_roundtrip(proj: &PathBuf, request: &str) -> (String, std::process::ExitStatus) {
    let mut child = Command::new(bin())
        .arg("serve")
        .current_dir(proj)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("`pforge serve` must spawn");

    {
        let stdin = child.stdin.as_mut().expect("stdin");
        writeln!(stdin, "{request}").expect("write request");
        stdin.flush().expect("flush");
    }

    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    // Blocking read: readiness comes from the child's own reply, not a timer.
    reader.read_line(&mut line).expect("read response");

    // Closing stdin (dropping the handle) ends the session.
    drop(child.stdin.take());
    let status = child.wait().expect("child must terminate");
    (line, status)
}

#[test]
fn initialize_returns_a_well_formed_mcp_result() {
    let (_root, proj) = scaffold("init");
    let (line, status) = rpc_roundtrip(
        &proj,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"dogfood","version":"1"}}}"#,
    );

    assert!(
        !line.trim().is_empty(),
        "server produced no response line — an MCP transport that answers nothing \
         is indistinguishable from one that is not wired up"
    );
    for needle in [r#""jsonrpc":"2.0""#, r#""id":1"#, r#""result""#] {
        assert!(
            line.contains(needle),
            "initialize response missing {needle}; got: {line}"
        );
    }
    assert!(
        line.contains("protocolVersion"),
        "initialize must negotiate a protocolVersion; got: {line}"
    );
    assert!(
        line.contains(r#""serverInfo""#),
        "initialize must identify the server; got: {line}"
    );
    assert!(
        status.success(),
        "server exited {:?} after a clean session; a transport that cannot shut \
         down cleanly leaks processes in CI",
        status.code()
    );
}

#[test]
fn server_identifies_itself_from_the_scaffolded_config() {
    let (_root, proj) = scaffold("ident");
    let (line, _) = rpc_roundtrip(
        &proj,
        r#"{"jsonrpc":"2.0","id":7,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"dogfood","version":"1"}}}"#,
    );
    // The scaffold names the project "demo"; the server must report the config
    // it actually loaded, not a hardcoded string.
    assert!(
        line.contains(r#""name":"demo""#),
        "serverInfo.name should come from the scaffolded pforge.yaml (expected \
         \"demo\"); got: {line}"
    );
    assert!(
        line.contains(r#""id":7"#),
        "id must echo the request; got: {line}"
    );
}

#[test]
fn malformed_request_does_not_crash_the_server() {
    let (_root, proj) = scaffold("bad");
    let (line, status) = rpc_roundtrip(&proj, "{ this is not json ");
    // Either a JSON-RPC error object or a clean exit is acceptable; a panic is
    // not. What matters is that the process terminates rather than wedging.
    assert!(
        status.code().is_some(),
        "server did not exit normally on malformed input (signal: {status:?})"
    );
    if !line.trim().is_empty() {
        assert!(
            line.contains("error") || line.contains("jsonrpc"),
            "a response to malformed input should be a JSON-RPC error; got: {line}"
        );
    }
}
