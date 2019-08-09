#!/usr/bin/env bash
set -euxo pipefail


MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri)
echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
rustup default "$MIRI_NIGHTLY"

rustup component add miri
cargo miri setup
cargo clean
cargo miri test -- -- -Zunstable-options --exclude-should-panic