# AGENTS.md

Read [CONTEXT.md](CONTEXT.md) for vocabulary and architecture rules.

## Check

```bash
git config core.hooksPath .githooks
sh scripts/check.sh
```

`scripts/check.sh` is the same gate CI runs in `.github/workflows/lint.yml`. Install the non-Rust tools once with
`uv tool install typos taplo shellcheck-py actionlint-py zizmor`.

CI also runs weekly and against `beta` (non-blocking), so new upstream lints surface before they reach `stable`.

## Lint policy

Every lint is an error. Fix the code rather than silencing the lint.

| Layer | Where | Enforces |
| --- | --- | --- |
| rustc + clippy | `[workspace.lints]` in `Cargo.toml` | `all` + `pedantic`, no `unsafe`, no `unwrap`/`expect`/`panic` outside tests, no `as` casts, no `print!`/`dbg!`/`todo!`, docs on every public item |
| clippy | `clippy.toml` | Determinism: no `HashMap`/`HashSet`, wall clocks, `thread::sleep`/`spawn`, `env::var`, or locks |
| rustfmt / taplo | `rustfmt.toml`, `taplo fmt` | Rust and TOML formatting |
| `scripts/lint-arch.sh` | shell | Core never imports Hegel or `lang/*`; `lang/*` never redefines core types or adds a second RNG/PBT; no demos in `lang/*/src/lib.rs`; no `//` line comments; files under 500 lines |
| typos, shellcheck, actionlint, `zizmor --pedantic` | CI `repo hygiene` job | Spelling, shell scripts, workflow correctness and security |
| `.githooks/commit-msg` | git hook | Conventional commit subjects: `type(scope): Capitalized subject` |

When an exception is genuinely right, use `#[expect(lint, reason = "...")]` on the smallest item. `#[allow]` is
rejected by `clippy::allow_attributes`.

When a review catches a pattern that a rule could have caught, add the rule to `clippy.toml` or
`scripts/lint-arch.sh` in the same PR.

## Code

- No line comments. Use names, types, `///` docs, and named tests.
- Randomness comes from `SimRng`; time comes from the logical `Clock`.
- Ordered collections only (`BTreeMap`, `BTreeSet`, `Vec`), so seed replay is stable.
