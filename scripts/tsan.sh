#!/usr/bin/env bash
set -euxo pipefail

export RUSTFLAGS="-Z sanitizer=thread"
export RUST_TEST_THREADS=1
export TSAN_OPTIONS="suppressions=$(pwd)/tests/suppressions.txt"

cargo +nightly test --test tsan
cargo +nightly test --test tsan --release