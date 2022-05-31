#!/usr/bin/env bash

set -e

find target/debug/deps -regex '.*/main[^.]*' -delete

RUSTFLAGS='-C instrument-coverage' cargo test

rust-profdata merge -sparse default.profraw -o default.profdata

rust-cov report -Xdemangler=rustfilt $(find target/debug/deps -regex '.*/main[^.]*') \
    -instr-profile=default.profdata \
    --ignore-filename-regex='/.cargo/registry' \
    --ignore-filename-regex='library/std' \
    --ignore-filename-regex='tests/main.rs'

rm -rf htmlcov

rust-cov show -Xdemangler=rustfilt $(find target/debug/deps -regex '.*/main[^.]*') \
    -instr-profile=default.profdata \
    --ignore-filename-regex='/.cargo/registry' \
    --ignore-filename-regex='library/std' \
    --ignore-filename-regex='tests/main.rs' \
    -format=html -o htmlcov

rm default.profraw default.profdata
