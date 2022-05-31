#!/usr/bin/env bash

set -e

if [ -d "target/debug/deps" ]; then
  find target/debug/deps -regex '.*/[^.]*' -type f -delete
fi

RUSTFLAGS='-C instrument-coverage' cargo test

rust-profdata merge -sparse default.profraw -o default.profdata

rust-cov report -Xdemangler=rustfilt $(find target/debug/deps -regex '.*/[^.]*' -type f) \
    -instr-profile=default.profdata \
    --ignore-filename-regex='/.cargo/registry' \
    --ignore-filename-regex='library/std' \
    --ignore-filename-regex='tests/main.rs'

rust-cov export -Xdemangler=rustfilt $(find target/debug/deps -regex '.*/[^.]*' -type f) \
    -instr-profile=default.profdata \
    --ignore-filename-regex='/.cargo/registry' \
    --ignore-filename-regex='library/std' \
    --ignore-filename-regex='tests/main.rs' \
    -format=lcov > default.lcov

rm default.profraw default.profdata
