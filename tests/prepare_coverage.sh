#!/usr/bin/env bash

rust-profdata merge -sparse default.profraw -o default.profdata
rust-cov export -Xdemangler=rustfilt target/debug/lib_pydantic_core.dylib \
    -instr-profile=default.profdata \
    --ignore-filename-regex='/.cargo/registry' \
    --ignore-filename-regex='library/std' \
    -format=lcov > rust_coverage.lcov
