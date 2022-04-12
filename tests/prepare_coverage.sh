#!/usr/bin/env bash

rust-profdata merge -sparse rust_coverage.profraw -o rust_coverage.profdata
rust-cov export -Xdemangler=rustfilt $(ls pydantic_core/*.so) \
    -instr-profile=rust_coverage.profdata \
    --ignore-filename-regex='/.cargo/registry' \
    --ignore-filename-regex='library/std' \
    -format=lcov > rust_coverage.lcov
