#!/usr/bin/env sh
# Lists all Rust source files in the repo (excluding target, scripts, docs, etc.)
find .. -type f -name '*.rs' \
    -not -path '../target/*' \
    -not -path '../scripts/*' \
    -not -path '../docs/*' \
