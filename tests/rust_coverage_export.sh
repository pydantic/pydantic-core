#!/usr/bin/env bash

set -e

rust-profdata merge -sparse *.profraw -o default.profdata

rust-cov report -Xdemangler=rustfilt pydantic_core/*.so \
    -instr-profile=default.profdata \
    --ignore-filename-regex='\.cargo/registry' \
    --ignore-filename-regex='library/std' \

rust-cov export -Xdemangler=rustfilt pydantic_core/*.so \
    -instr-profile=default.profdata \
    --ignore-filename-regex='\.cargo/registry' \
    --ignore-filename-regex='library/std' \
    -format=lcov > default.lcov

rm *.profraw default.profdata
