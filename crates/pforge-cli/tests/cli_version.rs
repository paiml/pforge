//! `pforge --version` must work.
//!
//! It did not. clap's derive only emits a `--version` flag when the command
//! declares one, and the parser declared `name` and `about` but not `version`:
//!
//!     $ pforge --version
//!     error: unexpected argument '--version' found
//!     Usage: pforge <COMMAND>
//!
//! This is not a cosmetic gap. `--version` is how tooling establishes that an
//! installed binary is both present AND the expected build — paiml/infra's
//! clean-room gate A4 runs it, and forjar's cargo package resource verifies
//! every managed tool by running `<bin> --version`, so a CLI without it can
//! never be managed as a forjar stack tool. It also means `cargo install`
//! leaves users with no way to ask what they installed.
//!
//! Asserting on the VERSION STRING, not merely on exit status: a `--version`
//! that prints nothing useful would satisfy a status-only check while still
//! being useless to the tooling that consumes it.

use std::process::Command;

fn pforge() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pforge"))
}

#[test]
fn version_flag_is_accepted_and_prints_the_crate_version() {
    let out = pforge().arg("--version").output().expect("pforge runs");
    assert!(
        out.status.success(),
        "`pforge --version` failed.\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "`--version` printed {stdout:?}, which does not contain the crate version {:?}",
        env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn short_version_flag_also_works() {
    let out = pforge().arg("-V").output().expect("pforge runs");
    assert!(
        out.status.success(),
        "`pforge -V` failed — clap provides both forms together, so this failing \
         while --version passes would mean the flag was hand-rolled rather than \
         declared.\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The counter-case: adding a version flag must not disturb the subcommand
/// interface. Without this, deleting `Commands` entirely would pass the tests
/// above.
#[test]
fn subcommands_still_work() {
    let out = pforge().arg("--help").output().expect("pforge runs");
    assert!(out.status.success(), "`pforge --help` failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("new") || stdout.contains("Commands"),
        "`--help` no longer lists subcommands:\n{stdout}"
    );
}
