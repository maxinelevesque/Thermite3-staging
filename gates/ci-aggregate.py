#!/usr/bin/env python3
"""Fail closed unless every supplied GitHub Actions child result succeeded."""

import argparse


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("gate")
    parser.add_argument("results", nargs="+")
    args = parser.parse_args()
    failures = [result for result in args.results if result != "success"]
    if failures:
        print(f"{args.gate} child result(s): {', '.join(args.results)}")
        return 1
    print(f"{args.gate} aggregate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
