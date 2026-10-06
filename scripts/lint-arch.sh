#!/bin/sh
# Architecture and pattern rules that clippy cannot express. Each rule maps to CONTEXT.md or AGENTS.md.

set -u

VIOLATIONS="$(mktemp "${TMPDIR:-/tmp}/lint-arch.XXXXXX")" || exit 1
trap 'rm -f "$VIOLATIONS"' EXIT INT TERM

report() {
  echo "$1:$2" >>"$VIOLATIONS"
  printf '[ARCH] %s:%s\n' "$1" "$2"
  printf '  Violation: %s\n' "$3"
  printf '  Fix: %s\n' "$4"
}

rust_files() {
  git ls-files -- '*.rs'
}

# 1) Core never depends on Hegel or on lang/* (CONTEXT.md rule 2).
grep -nE '^\s*(hegel|hegeltest|openchaos)\s*=' crates/openchaos-core/Cargo.toml | while IFS=: read -r line _; do
  report crates/openchaos-core/Cargo.toml "$line" \
    "openchaos-core depends on Hegel or a language package." \
    "Move the Hegel-facing code into lang/<language>; core stays dependency-free."
done
git ls-files -- 'crates/openchaos-core/*.rs' | xargs grep -nE '\b(hegel|openchaos)::' 2>/dev/null | while IFS=: read -r file line _; do
  report "$file" "$line" \
    "openchaos-core references Hegel or a language package." \
    "Core owns the sim only; adapt it from lang/<language>."
done

# 2) Language packages never reimplement core sim types (CONTEXT.md rule 1).
git ls-files -- 'lang/*.rs' | xargs grep -nE '^\s*(pub(\([a-z]+\))?\s+)?(struct|enum|trait)\s+(Clock|Scheduler|SimWorld|SimRng|Seed|Meter|Span|RegionStats|SimReport|TimedEvent|EventId|RunLimit)\b' 2>/dev/null | while IFS=: read -r file line _; do
  report "$file" "$line" \
    "Language package defines a type that core already owns." \
    "Re-export or bind the openchaos-core type instead of a second implementation."
done
git ls-files -- 'lang/*.rs' | xargs grep -nE '\b(rand|fastrand|getrandom|proptest|quickcheck)::' 2>/dev/null | while IFS=: read -r file line _; do
  report "$file" "$line" \
    "Language package uses a second entropy or PBT source." \
    "Draw seeds through Hegel (openchaos::bind) and randomness through SimRng."
done

# 3) Demo entrypoints stay out of language package roots (CONTEXT.md rule 4).
for lib in $(git ls-files -- 'lang/*/src/lib.rs'); do
  grep -nE '^\s*(pub\s+)?(mod|use)\s+.*\b(demo|example|examples|lru|bench_demo)\b' "$lib" | while IFS=: read -r line _; do
    report "$lib" "$line" \
      "Language package root exposes a demo." \
      "Keep demos in examples/ or tests/ and call the adapter from there."
  done
done

# 4) No line comments. Names, types, doc comments, and tests carry intent.
rust_files | xargs grep -nE '^\s*//([^/!]|$)|[^:/"]//\s' 2>/dev/null | grep -vE '//\s*SAFETY:' | while IFS=: read -r file line _; do
  report "$file" "$line" \
    "Line comment in Rust source." \
    "Delete it, rename to make it obvious, move it into a /// doc comment, or encode it as a named test."
done

# 5) Production files stay small enough to read in one sitting.
rust_files | grep -v '/tests/' | while IFS= read -r file; do
  lines=$(wc -l <"$file" | tr -d ' ')
  if [ "$lines" -gt 500 ]; then
    report "$file" "$lines" \
      "File exceeds 500 lines ($lines)." \
      "Split it by concern into focused modules."
  fi
done

# 6) Unsafe code lives only in the Valgrind client-request shim.
rust_files | grep -v '^crates/openchaos-bench/src/hooks.rs$' | xargs grep -nE '\bunsafe\b' 2>/dev/null | while IFS=: read -r file line _; do
  report "$file" "$line" \
    "unsafe outside crates/openchaos-bench/src/hooks.rs." \
    "Use a safe API; only the Valgrind client-request asm needs unsafe."
done

count=$(wc -l <"$VIOLATIONS" | tr -d ' ')
if [ "$count" -gt 0 ]; then
  printf '\nlint-arch: %s violation(s)\n' "$count"
  exit 1
fi
