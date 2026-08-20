#!/usr/bin/env python3
"""Drive a pforge MCP stdio server deterministically and report what it does.

WHY THIS IS NOT SHELL
--------------------
The first version of dogfood-use.sh piped requests into `pforge serve` and
redirected stdout to a file. That is a race, and it was measured losing data:
the identical 3-message conversation produced 2 response lines when stdout was
a pipe and 0 lines when it was a file. When stdout is not a tty the server
block-buffers, so if `timeout` kills it before the buffer flushes the responses
are gone -- and a gate that reports "server produced no output" for a server
that answered fine is worse than no gate, because it is red for the wrong
reason and teaches people to ignore it.

So the conversation happens here: write a request, block on reading its
response line, then move on. Readiness comes from the child's own reply, never
from a sleep.

Usage:  mcp_probe.py <binary> <project-dir>
Prints one JSON object on stdout. Exit 0 means the probe ran, NOT that the
server behaved -- read the JSON for that.
"""
import json
import subprocess
import sys
import threading

INIT = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "dogfood", "version": "1"},
    },
}
# Required before tools/list will answer. Omitting it returns nothing at all,
# which is indistinguishable from a dead server.
INITIALIZED = {"jsonrpc": "2.0", "method": "notifications/initialized"}


def _spawn(binary, project):
    return subprocess.Popen(
        [binary, "serve"],
        cwd=project,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )


def _drain(stream, sink):
    def run():
        try:
            for line in stream:
                sink.append(line.rstrip())
        except Exception:
            pass

    t = threading.Thread(target=run, daemon=True)
    t.start()
    return t


def _read_reply(proc):
    line = proc.stdout.readline()
    if not line:
        return None
    try:
        return json.loads(line)
    except json.JSONDecodeError:
        return {"_unparseable": line.strip()}


def _exchange(proc, msg, replies):
    """Send one message; block for its reply only if it carries an id."""
    proc.stdin.write(json.dumps(msg) + "\n")
    proc.stdin.flush()
    if "id" not in msg:
        return  # a notification has no reply; do not block on one
    reply = _read_reply(proc)
    if reply is not None:
        replies[msg["id"]] = reply


def _shutdown(proc):
    try:
        proc.stdin.close()
    except Exception:
        pass
    try:
        proc.wait(timeout=15)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()


def converse(binary, project, messages):
    """Send messages in order; return (replies-by-id, exit code, stderr)."""
    proc = _spawn(binary, project)
    replies, err = {}, []
    _drain(proc.stderr, err)
    try:
        for msg in messages:
            _exchange(proc, msg, replies)
    except BrokenPipeError:
        pass
    finally:
        _shutdown(proc)
    return replies, proc.returncode, err


def _classify(reply):
    """REACHABLE / NOTFOUND / NORESPONSE for one tools/call reply."""
    if reply is None:
        return "NORESPONSE"
    if "error" not in reply:
        return "REACHABLE"
    message = str((reply["error"] or {}).get("message", "")).lower()
    # "not found" means list and call disagree: the advertised surface is not
    # real. Any OTHER error (bad arguments, handler failure) means the tool IS
    # reachable, which is what this gate asserts.
    return "NOTFOUND" if "not found" in message else "REACHABLE"


def _call_tool(binary, project, name):
    replies, _, _ = converse(
        binary,
        project,
        [
            INIT,
            INITIALIZED,
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {"name": name, "arguments": {}},
            },
        ],
    )
    return _classify(replies.get(3))


def _list_tools(replies):
    return ((replies.get(2) or {}).get("result") or {}).get("tools") or []


def main():
    binary, project = sys.argv[1], sys.argv[2]

    replies, rc, err = converse(
        binary,
        project,
        [INIT, INITIALIZED, {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}],
    )
    tools = _list_tools(replies)

    # One fresh server per call, so a crash on one tool cannot mask the next.
    calls = {}
    for tool in tools:
        name = tool.get("name")
        if name:
            calls[name] = _call_tool(binary, project, name)

    print(
        json.dumps(
            {
                "initialize": replies.get(1),
                "exit_code": rc,
                "stderr": err[:8],
                "tools": tools,
                "calls": calls,
            }
        )
    )


if __name__ == "__main__":
    main()
