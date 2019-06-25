#!/usr/bin/env bash

set -euo pipefail

# we're using unstable config so we need nightly compiler
cargo +nightly fmt --all -- "$@"
