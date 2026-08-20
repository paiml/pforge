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
# A test you author cannot falsify a premise you hold. pforge's suites were
# green while `pforge new` + `pforge serve` — the exact two commands `pforge
# new` prints under "Next steps" — produced a server that advertised a tool it
# could not call (paiml/pforge#12, #13). Those tests share an author with the
# code, so they confirmed the workflow rather than ran it. This runs it.
#
# THE INVARIANT, CHECKED FROM BOTH SIDES
#
#   Every name returned by tools/list MUST be callable via tools/call.
#
#   A. A `cli` tool IS registered by the generic binary, so the server must
#      start, advertise it, dispatch it, and publish a non-empty inputSchema.
#   B. A `native` tool is NOT registered by the generic binary, so the server
#      must REFUSE TO START rather than advertise something it cannot dispatch.
#
# Checking only (A) would let the old bug back in the moment someone re-adds an
# adapter for unregistered tools; checking only (B) would pass on a server that
# refuses everything. Both, or neither means anything.
#
# THE CONVERSATION IS DRIVEN BY scripts/mcp_probe.py, NOT BY SHELL.
# The first cut piped requests in with stdout redirected to a file. Measured:
# the identical 3-message conversation yielded 2 response lines to a pipe and 0
# to a file — the server block-buffers when stdout is not a tty, and `timeout`
# killed it before the flush. A gate that reports "no output" for a server that
# answered correctly is red for the wrong reason, and those get ignored.
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

# ── A. A dispatchable tool must round-trip completely ───────────────────────
PROJ_A="$WORK/dispatchable"
mkdir -p "$PROJ_A"
cat > "$PROJ_A/pforge.yaml" <<'YAML'
forge:
  name: dogfood-cli
  version: 0.1.0
  transport: stdio

tools:
  - type: cli
    name: echo_it
    description: "Echo a fixed string"
    command: echo
    args: ["dogfood-ok"]
YAML

REPORT="$WORK/mcp-report.json"
python3 "$HERE/mcp_probe.py" "$BIN" "$PROJ_A" > "$REPORT" 2>"$WORK/probe.err" \
  || fail "mcp_probe.py crashed: $(head -3 "$WORK/probe.err" 2>/dev/null)"
[ -s "$REPORT" ] || fail "probe produced no report"

python3 - "$REPORT" <<'PY'
import json, sys

report = json.load(open(sys.argv[1]))
problems = []

if not report.get("initialize"):
    problems.append("server never answered `initialize` — the MCP transport is not reachable")

tools = report.get("tools") or []
if not tools:
    problems.append("tools/list returned no tools; every assertion below would be vacuous")

for name, verdict in (report.get("calls") or {}).items():
    if verdict == "NOTFOUND":
        problems.append(
            f"tools/list advertises '{name}' but tools/call reports it does not exist "
            f"(paiml/pforge#12) — an MCP client reads tools/list and believes it"
        )
    elif verdict == "NORESPONSE":
        problems.append(f"tools/call '{name}' produced no response at all")

# The published schema is the client's ONLY contract for calling a tool.
for t in tools:
    if not ((t.get("inputSchema") or {}).get("properties") or {}):
        problems.append(
            f"'{t.get('name','?')}' publishes an empty inputSchema.properties "
            f"(paiml/pforge#13) — an LLM given properties:{{}} cannot know what to pass"
        )

if problems:
    print("DOGFOOD FAIL:", file=sys.stderr)
    for p in problems:
        print(f"  · {p}", file=sys.stderr)
    sys.exit(1)

print("  A: advertised and callable: " + ", ".join(t.get("name", "?") for t in tools))
PY

# ── B. An UNdispatchable tool must be refused, not advertised ───────────────
PROJ_B_ROOT="$WORK/native"
rm -rf "$PROJ_B_ROOT"; mkdir -p "$PROJ_B_ROOT"
( cd "$PROJ_B_ROOT" && "$BIN" new demo >/dev/null 2>&1 ) \
  || fail "\`pforge new demo\` failed — the scaffold is the fixture for (B)"
PROJ_B="$PROJ_B_ROOT/demo"
[ -f "$PROJ_B/pforge.yaml" ] || fail "\`pforge new\` produced no pforge.yaml"

set +e
REFUSAL=$( cd "$PROJ_B" && printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
           | "$BIN" serve 2>&1 >/dev/null )
RC=$?
set -e

if [ "$RC" -eq 0 ]; then
  fail "\`pforge serve\` STARTED on a scaffold whose native handler it cannot dispatch. It would advertise a tool that always fails on call (paiml/pforge#12)."
fi
case "$REFUSAL" in
  *"refusing to start"*) : ;;
  *) fail "serve exited $RC but without the explicit refusal — an operator cannot act on an unexplained failure. Got: $(echo "$REFUSAL" | tail -2)" ;;
esac
case "$REFUSAL" in
  *"pforge build"*) : ;;
  *) fail "the refusal does not say how to fix it (build the project's own binary)" ;;
esac
note "B: native scaffold correctly refused, naming the tool and the remedy"

echo "dogfood-use: pforge round-tripped its own output through MCP, both sides of the invariant"
