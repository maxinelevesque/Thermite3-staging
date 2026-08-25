#!/usr/bin/env python3
"""Run a command and always emit a small machine-readable timing record."""

import argparse
import json
import subprocess
import time
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--label", required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command:
        parser.error("a command is required")

    started = time.monotonic()
    result = subprocess.run(command, check=False)
    record = {
        "schema": 1,
        "label": args.label,
        "elapsed_seconds": round(time.monotonic() - started, 3),
        "exit_code": result.returncode,
        "command": command,
    }
    args.out.write_text(json.dumps(record, sort_keys=True) + "\n")
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
