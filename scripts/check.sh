#!/bin/sh
set -eu

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
sh scripts/lint-arch.sh
typos
taplo fmt --check
shellcheck scripts/*.sh .githooks/*
actionlint
zizmor --pedantic --offline .github/workflows
