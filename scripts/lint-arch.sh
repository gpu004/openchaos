#!/bin/sh
# Architecture and pattern rules that clippy cannot express.

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

# 1) Core never depends on Hegel or on lang/*, so any language package can wrap it.
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

# 2) Language packages never reimplement core sim types or add a second entropy source.
git ls-files -- 'lang/*.rs' | xargs grep -nE '^\s*(pub(\([a-z]+\))?\s+)?(struct|enum|trait)\s+(Clock|Scheduler|SimWorld|SimRng|Seed|TimedEvent|EventId|InPast|EventHandler|RunSummary)\b' 2>/dev/null | while IFS=: read -r file line _; do
  report "$file" "$line" \
    "Language package defines a type that core already owns." \
    "Re-export or bind the openchaos-core type instead of a second implementation."
done
git ls-files -- 'lang/*.rs' | xargs grep -nE '\b(rand|fastrand|getrandom|proptest|quickcheck)::' 2>/dev/null | while IFS=: read -r file line _; do
  report "$file" "$line" \
    "Language package uses a second entropy or PBT source." \
    "Draw seeds through Hegel (openchaos::draw_seed) and randomness through SimRng."
done

# 3) No line comments. Names, types, doc comments, and tests carry intent.
rust_files | xargs grep -nE '^\s*//([^/!]|$)|[^:/"]//\s' 2>/dev/null | grep -vE '//\s*SAFETY:' | while IFS=: read -r file line _; do
  report "$file" "$line" \
    "Line comment in Rust source." \
    "Delete it, rename to make it obvious, move it into a /// doc comment, or encode it as a named test."
done

# 4) Production files stay small enough to read in one sitting.
rust_files | grep -v '/tests/' | while IFS= read -r file; do
  lines=$(wc -l <"$file" | tr -d ' ')
  if [ "$lines" -gt 500 ]; then
    report "$file" "$lines" \
      "File exceeds 500 lines ($lines)." \
      "Split it by concern into focused modules."
  fi
done

count=$(wc -l <"$VIOLATIONS" | tr -d ' ')
if [ "$count" -gt 0 ]; then
  printf '\nlint-arch: %s violation(s)\n' "$count"
  exit 1
fi
