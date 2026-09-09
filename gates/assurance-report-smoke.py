#!/usr/bin/env python3
"""Exercise the mixed-route assurance report through the public CLI."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REVISION = "a" * 40


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="thermite-assurance-") as directory:
        output = Path(directory)
        report_path = output / "report.json"
        html_path = output / "report.html"
        result = subprocess.run(
            [
                "cargo",
                "run",
                "--quiet",
                "--locked",
                "-p",
                "forge",
                "--",
                "assurance",
                "conformance/forge/mix64.th",
                "--revision",
                REVISION,
                "--trust",
                "local",
                "--out-json",
                str(report_path),
                "--out-html",
                str(html_path),
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
            timeout=180,
        )
        if result.returncode != 0:
            return 4
        try:
            report = json.loads(report_path.read_text(encoding="utf-8"))
            html = html_path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError, json.JSONDecodeError):
            return 4

        project = report["body"]["project"]
        if project["population"]["covered"] != 1 or project["population"]["total"] != 2:
            return 4
        if "Boundary: end to end" not in project["headline"] or "L3" in project["headline"]:
            return 4
        if not any(
            exception["identity"]["item_path"] == "rotl1_injective"
            for exception in project["exceptions"]
        ):
            return 4
        mix64 = next(
            item
            for item in report["body"]["items"]
            if item["identity"]["item_path"] == "mix64"
        )
        procedures = {
            clause["certification"]["procedure"]["kind"]
            for clause in mix64["clauses"]
        }
        if len(mix64["clauses"]) != 3 or procedures != {"bit_vector", "nlsat"}:
            return 4
        if (
            "Common project claims" not in html
            or "Complete formal JSON portrait" not in html
        ):
            return 4
        if report["report_sha256"] not in html or REVISION not in html:
            return 4
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
