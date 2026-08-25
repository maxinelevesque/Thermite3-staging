#!/usr/bin/env bash
# Gate G3: parse both release configurations, exercise the real BV route, and
# replay the resulting validity theorems in Lean. Each named segment is a
# directly runnable CI unit; `all` preserves the historical aggregate gate.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

SEGMENT="${1:-all}"
case "$SEGMENT" in
  all|parser-lowering|checked-replay) ;;
  *)
    echo "usage: $0 [all|parser-lowering|checked-replay]" >&2
    exit 2
    ;;
esac

for tool in cargo; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "g3-gate: required tool not found: $tool" >&2
    exit 2
  }
done

run_parser_lowering() {
  echo "[G3 parser-lowering 1/3] release parser without bv plumbing"
  cargo test -p thermite-syntax --release --test bv_tag_parse

  echo "[G3 parser-lowering 2/3] release parser with bv plumbing"
  cargo test -p thermite-syntax --release --features bv --test bv_tag_parse

  echo "[G3 parser-lowering 3/3] fixed-width lowering and invariant preservation"
  cargo test -p thermite-lower tagged_ -- --nocapture
  cargo test -p forge --features bv --test bv_invariants -- --nocapture
}

run_checked_replay() {
  for tool in lake verus z3; do
    command -v "$tool" >/dev/null 2>&1 || {
      echo "g3-gate: required tool not found: $tool" >&2
      exit 2
    }
  done

  echo "[G3 checked-replay 1/2] Lean spine and reconstruction axiom probe"
  bash gates/lean-axiom-probe.sh

  echo "[G3 checked-replay 2/2] checked validity replay and the live BV route"
  cargo test -p forge --features bv --bin forge lean_smt_export::tests -- --nocapture
  cargo test -p forge --features bv --bin forge \
    check::tests::req8_arithmetic_and_bitwise_clauses_migrate_to_kernel_checked \
    -- --nocapture
  cargo test -p forge --features bv --test bv_lowering -- --nocapture
}

case "$SEGMENT" in
  all)
    run_parser_lowering
    run_checked_replay
    ;;
  parser-lowering) run_parser_lowering ;;
  checked-replay) run_checked_replay ;;
esac

echo "G3 $SEGMENT gate passed"
