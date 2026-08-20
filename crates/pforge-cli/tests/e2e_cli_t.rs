//! End-to-end verification of the CLI transport, against the SHIPPED BINARY.
//!
//! Every test here spawns `CARGO_BIN_EXE_pforge`. That is the point, and it is
//! load-bearing: a suite that calls the library cannot see whether a subcommand
//! is reachable from `main`. rmedia's four-way parity suite was green for the
//! entire period its MCP and HTTP transports had no caller from `main.rs` —
//! the transports agreed with each other perfectly and were unreachable from
//! the process entry point.
//!
//! Declared in Cargo.toml as `[package.metadata.transports] cli`.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pforge"))
}

/// The subcommands the binary advertises in its own `--help`, read from the
/// artifact rather than hand-listed here. A hand-written fixture drifts; this
/// cannot, because it is the same text a user reads.
fn advertised_subcommands() -> Vec<String> {
    let out = bin()
        .arg("--help")
        .output()
        .expect("pforge --help must run");
    assert!(out.status.success(), "--help must exit 0");
    let help = String::from_utf8_lossy(&out.stdout);

    let mut in_commands = false;
    let mut names = Vec::new();
    for line in help.lines() {
        if line.starts_with("Commands:") {
            in_commands = true;
            continue;
        }
        if in_commands {
            // The block ends at the first blank line or the Options: header.
            if line.trim().is_empty() || line.starts_with("Options:") {
                break;
            }
            if let Some(name) = line.split_whitespace().next() {
                if name != "help" {
                    names.push(name.to_string());
                }
            }
        }
    }
    assert!(
        !names.is_empty(),
        "parsed zero subcommands from --help — a suite that checks nothing is a \
         vacuous pass, not a clean one. Help text was:\n{help}"
    );
    names
}

#[test]
fn every_advertised_subcommand_answers_help() {
    let subs = advertised_subcommands();
    for sub in &subs {
        let out = bin()
            .args([sub, "--help"])
            .output()
            .unwrap_or_else(|e| panic!("spawning `pforge {sub} --help` failed: {e}"));
        assert!(
            out.status.success(),
            "`pforge {sub} --help` exited {:?} — the binary advertises a subcommand \
             it cannot service.\nstderr: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    // Guard the guard: if --help ever stops listing commands, the loop above
    // would pass by iterating nothing.
    assert!(subs.len() >= 4, "expected >=4 subcommands, got {subs:?}");
}

#[test]
fn version_flag_reports_the_crate_version() {
    let out = bin().arg("--version").output().expect("--version must run");
    assert!(out.status.success(), "--version must exit 0");
    let printed = String::from_utf8_lossy(&out.stdout);
    assert!(
        printed.contains(env!("CARGO_PKG_VERSION")),
        "`--version` printed {printed:?} which does not contain the compiled-in \
         version {}. This is how the binary identifies itself to forjar's cargo \
         package resource.",
        env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn unknown_subcommand_fails_loudly() {
    let out = bin()
        .arg("definitely-not-a-subcommand")
        .output()
        .expect("must run");
    assert!(
        !out.status.success(),
        "an unknown subcommand must NOT exit 0 — silent acceptance is how a typo \
         becomes a no-op that looks like success"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.trim().is_empty(),
        "an unknown subcommand must explain itself on stderr"
    );
}
