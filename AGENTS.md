# AGENTS.md

## Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Skills

Skills live in `.agents/skills/`. Run them on every change, not only when asked:

| When | Skill |
| --- | --- |
| Before opening or updating a PR, on the branch diff | `deslop`, then `no-comments` |
| Before opening a PR that adds or restructures modules | `thermo-nuclear-code-quality-review` |
| Any prose you write: README, ADRs, PR descriptions, doc comments | `unslop` |
| The operator corrects a mistake an agent already made once | `correct` |
| Questions about how or why existing code works | `how`, `why` |
