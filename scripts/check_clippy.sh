#!/usr/bin/env sh
# Runs clippy linter for Rust
cd ..
cargo clippy --all-targets --all-features -- -D warnings
