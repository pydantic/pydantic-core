#!/usr/bin/env bash

set -ex

rust-profdata merge -sparse *.profraw -o default.profdata

rust-cov report -Xdemangler=rustfilt $(ls pydantic_core/*.so) \
    -instr-profile=default.profdata \
    --ignore-filename-regex='/.cargo/registry' \
    --ignore-filename-regex='library/std' \

rust-cov export -Xdemangler=rustfilt $(ls pydantic_core/*.so) \
    -instr-profile=default.profdata \
    --ignore-filename-regex='/.cargo/registry' \
    --ignore-filename-regex='library/std' \
    -format=lcov > default.lcov

rm *.profraw default.profdata
