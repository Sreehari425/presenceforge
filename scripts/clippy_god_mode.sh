#!/usr/bin/env sh
cd .. # assuming you are running from scripts directory
cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::nursery
