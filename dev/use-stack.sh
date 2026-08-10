#!/usr/bin/env bash
# use-stack.sh — materialize an opt-in stack into the working tree.
#
# RFC-15 §3.2. A stack under opt-in/<name>/ MIRRORS the repository root, so
# materializing is a structure-preserving copy and there is no manifest to keep
# in sync with the files it describes.
#
# Usage:
#   dev/use-stack.sh                 list stacks, and which are installed
#   dev/use-stack.sh <name>          materialize <name>
#   dev/use-stack.sh <name> --force  materialize, overwriting differing files
#
# Refuses by default when a target exists and DIFFERS from the stack, printing
# the diff. That is deliberate and is the whole of the RFC-15 §4 "Stack drift"
# item made visible: during the step 1..3 window the tracked config and the
# stack are two copies of one thing, and a silent overwrite would discard
# whichever one someone edited. Skipping instead would serve a stale tree, which
# is a check that fails open — ruled against in this repo more than once.
#
# Exit codes:
#   0  materialized, or already identical, or listing
#   1  refused: a target differs and --force was not given
#   2  usage error (no such stack)
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STACKS="$ROOT/opt-in"

list_stacks() {
  printf 'Available stacks (opt-in/):\n'
  local any=0
  for d in "$STACKS"/*/; do
    [ -d "$d" ] || continue
    local name files state
    name="$(basename "$d")"
    files="$(find "$d" -type f ! -name 'README.md' | wc -l | tr -d ' ')"
    if [ "$files" -eq 0 ]; then
      state="empty — nothing to materialize yet"
    elif stack_is_installed "$name"; then
      state="installed"
    else
      state="not installed"
    fi
    printf '  %-12s %-3s file(s)   %s\n' "$name" "$files" "$state"
    any=1
  done
  [ "$any" -eq 1 ] || printf '  (none)\n'
  printf '\nInstall one with:  just use <name>\n'
}

# A stack counts as installed when every file it carries exists at the root.
# Content is not compared here: "installed but drifted" is a real state and is
# reported by the materialize path, which prints the diff.
stack_is_installed() {
  local name="$1" src="$STACKS/$1" rel
  local found=0
  while IFS= read -r f; do
    rel="${f#"$src"/}"
    [ "$rel" = "README.md" ] && continue
    found=1
    [ -e "$ROOT/$rel" ] || return 1
  done < <(find "$src" -type f)
  [ "$found" -eq 1 ]
}

materialize() {
  # Declared separately on purpose: `local a=$1 b=$STACKS/$a` declares both
  # names before assigning either, so $a is unbound when b is evaluated under
  # `set -u`.
  local name="$1"
  local force="$2"
  local src="$STACKS/$name"
  [ -d "$src" ] || { printf 'error: no stack %s under opt-in/\n' "$name" >&2; list_stacks >&2; exit 2; }

  # Newline-delimited rather than arrays: bash 3.2 — still the /bin/bash on
  # macOS — treats "${arr[@]}" on an EMPTY array as an unbound variable under
  # `set -u`, so the no-op and empty-stack paths would both abort. Stack paths
  # are repo-relative and contain no newlines.
  local all="" differing="" ndiff=0 rel
  while IFS= read -r f; do
    rel="${f#"$src"/}"
    [ "$rel" = "README.md" ] && continue
    all="$all$rel"$'\n'
    if [ -e "$ROOT/$rel" ] && ! cmp -s "$f" "$ROOT/$rel"; then
      differing="$differing$rel"$'\n'
      ndiff=$((ndiff + 1))
    fi
  done < <(find "$src" -type f)

  if [ "$ndiff" -gt 0 ] && [ "$force" != "1" ]; then
    printf 'error: %s file(s) differ from the %s stack:\n\n' "$ndiff" "$name" >&2
    while IFS= read -r rel; do
      [ -z "$rel" ] && continue
      printf '  %s\n' "$rel" >&2
      diff -u "$ROOT/$rel" "$src/$rel" | sed '1,2d;s/^/    /' >&2 || true
    done <<< "$differing"
    printf '\nThe stack and the working tree disagree. Decide which is right:\n' >&2
    printf '  reconcile opt-in/%s/ to match, or re-run with --force to overwrite.\n' "$name" >&2
    exit 1
  fi

  local n=0
  while IFS= read -r rel; do
    [ -z "$rel" ] && continue
    mkdir -p "$ROOT/$(dirname "$rel")"
    cp "$src/$rel" "$ROOT/$rel"
    n=$((n + 1))
  done <<< "$all"
  if [ "$n" -eq 0 ]; then
    printf '%s: nothing to materialize (the stack is empty)\n' "$name"
  else
    printf '%s: materialized %s file(s)\n' "$name" "$n"
  fi
}

case "${1:-}" in
  "") list_stacks ;;
  -h|--help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//' ;;
  *)
    force=0
    [ "${2:-}" = "--force" ] && force=1
    materialize "$1" "$force"
    ;;
esac
