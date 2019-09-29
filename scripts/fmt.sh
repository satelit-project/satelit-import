#!/usr/bin/env bash

set -euo pipefail

# we're using unstable config so we need nightly compiler
toolchain=$(rustup toolchain list | grep -m 1 nightly)
cargo +${toolchain} fmt --all -- "$@"
