#!/usr/bin/env python3
"""Aggregate Pi-agent session friction for the autonomous implementation workflow.

Uses `terapia` (typed CLI over ~/.pi/agent/sessions) for the session inventory and
per-session error rows, and a fast rg pass over session files for shell-command
friction signals (cd re-establishment, sleep-polling, shell-based reads).

Outputs: JSON (machine) + markdown summary (human) with a per-repo breakdown and
waste expressed in common units (polling wall-clock minutes, wasted tool turns).

Selection predicate (single source of truth — review finding EDGE-002):
a session is included iff its START timestamp >= --since AND its project matches
the project filter. File-level mtime is never used for selection.

Usage:
    python3 scripts/session-friction.py --since 2026-10-13 \
        [--out /tmp/friction] [--project-regex 'gh/(charly|ak)/']

Terapia requirements: `terapia list` and `terapia errors <id>` must emit JSON.
Cache: terapia is mtime-keyed and incremental; run once before publishing.
"""
from __future__ import annotations

import argparse
import collections
import json
import re
import subprocess
import sys
from pathlib import Path

SESSIONS_DIR = Path.home() / ".pi" / "agent" / "sessions"

# ~15 tokens per redundant "cd <repo> && " prefix (review: token-waste estimate)
TOKENS_PER_CD = 15


def sh(cmd: list[str], timeout: int = 300) -> str:
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    return r.stdout


def load_json(proc_out: str):
    return json.loads(proc_out)


def parse_terapia_errors(text: str):
    """terapia errors output is per-session pretty-printed arrays; be lenient."""
    text = text.strip()
    if not text:
        return []
    if text.startswith("["):
        # concatenated per-session arrays: ][ -> ,[
        text = re.sub(r"\]\s*\[", ",", text)
        try:
            arr = json.loads(text, strict=False)
            return [r for r in arr if isinstance(r, dict)]
        except json.JSONDecodeError:
            pass
    rows = []
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("{") and line.endswith("}"):
            try:
                rows.append(json.loads(line, strict=False))
            except json.JSONDecodeError:
                continue
    return rows


def rg_count(files: list[str], pattern: str) -> int:
    if not files:
        return 0
    out = subprocess.run(
        ["rg", "-o", "--no-filename", "-N", "-e", pattern, *files],
        capture_output=True, text=True,
    )
    return sum(1 for _ in out.stdout.splitlines())


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--since", required=True, help="ISO date, e.g. 2026-10-13")
    # terapia project names are dash-joined cwd paths, e.g.
    # var-home-sasha-para-areas-dev-gh-charly-wai
    ap.add_argument("--project-regex", default=r"gh-(charly|ak)-|para-tmp-tambor")
    ap.add_argument("--min-tool-calls", type=int, default=20)
    ap.add_argument("--out", default="/tmp/session-friction")
    args = ap.parse_args()

    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    sessions = load_json(sh(["terapia", "list"]))
    sel = [
        s for s in sessions
        if s["timestamp"] >= args.since
        and re.search(args.project_regex, s["project"])
        and s["tool_calls"] >= args.min_tool_calls
    ]
    print(f"selected sessions: {len(sel)}", file=sys.stderr)

    # --- error rows via terapia (only sessions that report errors > 0) ---
    err_rows = []
    with_err = [s for s in sel if s["errors"] > 0]
    for i, s in enumerate(with_err):
        err_rows.extend(parse_terapia_errors(sh(["terapia", "errors", s["id"]])))
        if (i + 1) % 25 == 0:
            print(f"  errors fetched {i+1}/{len(with_err)}", file=sys.stderr)
    err_total_expected = sum(s["errors"] for s in sel)
    err_total_got = len(err_rows)
    consistency = {
        "expected": err_total_expected, "extracted": err_total_got,
        "ok": err_total_got >= err_total_expected,  # review EDGE-003
    }

    # --- friction signals via rg over the selected sessions' files ---
    files = [s["file"] for s in sel]
    signals = {
        "cd_prefixed_bash": rg_count(files, r'"command":"cd [/~]'),
        "sleep_poll_ge60": rg_count(files, r'"command":"sleep (60|[1-9][0-9]{2,})'),
        "shell_reads": rg_count(files, r'"command":"(grep|sed|awk|head|tail|cat) '),
    }
    # sleep seconds actually waited (sum of sleep durations in poll commands)
    sleep_secs = 0
    rg = subprocess.run(
        ["rg", "-o", "--no-filename", "-N", r'"command":"sleep ([0-9]+)', *files],
        capture_output=True, text=True,
    )
    for line in rg.stdout.splitlines():
        m = re.search(r"sleep (\d+)", line)
        if m and int(m.group(1)) >= 60:
            sleep_secs += int(m.group(1))

    # --- aggregates ---
    by_repo = collections.defaultdict(lambda: collections.Counter())
    for s in sel:
        repo = re.sub(r"^.*gh/", "gh/", s["project"]).rstrip("-")
        by_repo[repo]["sessions"] += 1
        by_repo[repo]["tool_calls"] += s["tool_calls"]
        by_repo[repo]["errors"] += s["errors"]
    cls = collections.Counter(r.get("class", "?") for r in err_rows)
    tool = collections.Counter(r.get("tool", "?") for r in err_rows)

    result = {
        "since": args.since,
        "selection": {
            "project_regex": args.project_regex,
            "min_tool_calls": args.min_tool_calls,
            "sessions": len(sel),
            "total_tool_calls": sum(s["tool_calls"] for s in sel),
            "consistency": consistency,
        },
        "by_repo": {
            repo: dict(c) for repo, c in
            sorted(by_repo.items(), key=lambda kv: -kv[1]["tool_calls"])
        },
        "error_classes": dict(cls.most_common()),
        "error_tools": dict(tool.most_common()),
        "signals": signals,
        "waste_units": {
            # polling wall-clock: every sleep>=60 in a poll loop blocks a turn
            "poll_wall_clock_minutes": round(sleep_secs / 60),
            # cd re-establishment: one redundant tool turn + ~15 tokens each
            "cd_wasted_turns": signals["cd_prefixed_bash"],
            "cd_wasted_tokens_est": signals["cd_prefixed_bash"] * TOKENS_PER_CD,
            # shell reads that duplicate the read tool
            "shell_read_turns": signals["shell_reads"],
        },
        "notes": [
            "Signals measured over selected sessions' files only (see selection).",
            "edit_mismatch↔model correlation is a HYPOTHESIS: sample is ~99% one model; no control cohort in-window.",
            "Waste units are estimates, not measurements, except poll_wall_clock_minutes.",
        ],
    }

    (out_dir / "friction.json").write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
