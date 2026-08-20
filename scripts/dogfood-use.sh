#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# scripts/dogfood-use.sh — pforge exercises its OWN generated output.
#
# Contract (sovereign dogfood protocol, gate 9): receives $BIN (the built
# release binary) and $WORK (a scratch dir), exits non-zero if the tool
# misbehaves on real input.
#
# WHY THIS EXISTS RATHER THAN MORE UNIT TESTS
#
# A test you author cannot falsify a premise you hold. pforge's suites are
# green while `pforge new` + `pforge serve` — the exact two commands `pforge
# new` prints under "Next steps" — produce a server that advertises a tool it
# cannot call (paiml/pforge#12). Those tests share an author with the code, so
# they confirmed the workflow rather than ran it.
#
# THE INVARIANT UNDER TEST
#
#   Every name returned by tools/list MUST be callable via tools/call.
#
# An MCP client — usually an LLM — reads tools/list and believes it. A tool
# that is advertised and then errors on call is worse than one never
# advertised: the failure surfaces to the model at use time as a confusing
# protocol error rather than as a missing capability.
#
# THE CONVERSATION IS DRIVEN BY scripts/mcp_probe.py, NOT BY SHELL.
# The first version of this script piped requests in and redirected stdout to a
# file. Measured: the identical 3-message conversation yielded 2 response lines
# to a pipe and 0 to a file, because the server block-buffers when stdout is
# not a tty and `timeout` killed it before the flush. A gate that reports "no
# output" for a server that answered correctly is red for the wrong reason, and
# a gate that is red for the wrong reason gets ignored.
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

BIN="${BIN:?dogfood-use: BIN (release binary) must be set}"
WORK="${WORK:?dogfood-use: WORK (scratch dir) must be set}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

fail() { echo "DOGFOOD FAIL: $*" >&2; exit 1; }
note() { echo "  $*"; }

command -v python3 >/dev/null 2>&1 \
  || fail "python3 absent — required to drive MCP. A dogfood run that skips its real dependency proves nothing."
[ -f "$HERE/mcp_probe.py" ] || fail "missing $HERE/mcp_probe.py"

note "binary: $BIN"
"$BIN" --version >/dev/null 2>&1 \
  || fail "\`$BIN --version\` does not work — forjar's cargo package resource identifies every managed tool this way"

# ── 1. Scaffold with the real binary ────────────────────────────────────────
PROJ_ROOT="$WORK/dogfood-scaffold"
rm -rf "$PROJ_ROOT"; mkdir -p "$PROJ_ROOT"
( cd "$PROJ_ROOT" && "$BIN" new demo >/dev/null 2>&1 ) \
  || fail "\`pforge new demo\` failed — the scaffold is the input to everything below"
PROJ="$PROJ_ROOT/demo"
[ -f "$PROJ/pforge.yaml" ] || fail "\`pforge new\` produced no pforge.yaml at $PROJ"
note "scaffolded: $PROJ"

# ── 2. Drive real MCP against the real server ───────────────────────────────
REPORT="$WORK/mcp-report.json"
python3 "$HERE/mcp_probe.py" "$BIN" "$PROJ" > "$REPORT" 2>"$WORK/probe.err" \
  || fail "mcp_probe.py crashed: $(head -3 "$WORK/probe.err" 2>/dev/null)"
[ -s "$REPORT" ] || fail "probe produced no report"

python3 - "$REPORT" "$PROJ/pforge.yaml" <<'PY'
import json, sys, re

report = json.load(open(sys.argv[1]))
yaml_text = open(sys.argv[2]).read()
problems = []

if not report.get("initialize"):
    problems.append("server never answered `initialize` — the MCP transport is not reachable at all")

tools = report.get("tools") or []
if not tools:
    problems.append("tools/list returned no tools; every assertion below would be vacuous")

# INVARIANT: everything advertised must be callable.
for name, verdict in (report.get("calls") or {}).items():
    if verdict == "NOTFOUND":
        problems.append(
            f"tools/list advertises '{name}' but tools/call reports it does not exist "
            f"(paiml/pforge#12) — an MCP client reads tools/list and believes it"
        )
    elif verdict == "NORESPONSE":
        problems.append(f"tools/call '{name}' produced no response at all")

# The published schema is the client's only contract for calling a tool.
if re.search(r"^\s*params:", yaml_text, re.M):
    empty = [
        t.get("name", "?")
        for t in tools
        if not ((t.get("inputSchema") or {}).get("properties") or {})
    ]
    if empty:
        problems.append(
            "pforge.yaml declares params but inputSchema.properties is empty for: "
            + ", ".join(empty)
            + " (paiml/pforge#13) — an LLM given properties:{} cannot know what to pass"
        )

if problems:
    print("DOGFOOD FAIL:", file=sys.stderr)
    for p in problems:
        print(f"  · {p}", file=sys.stderr)
    sys.exit(1)

print(f"  tools advertised and all callable: {', '.join(t.get('name','?') for t in tools)}")
PY

echo "dogfood-use: pforge round-tripped its own scaffold through MCP"
