#!/usr/bin/env sh
# Counts lines of Rust code in the repo (excluding target, scripts, docs, etc.)
find .. -type f -name '*.rs' \
    -not -path '../target/*' \
    -not -path '../scripts/*' \
    -not -path '../docs/*' \
    -not -path '../.history/' \

    | xargs wc -l | tail -n 1
