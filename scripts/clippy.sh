#!/usr/bin/env bash

set -euo pipefail

# fail even if there're compiler warnings
cargo clippy --all-targets --all-features -- -D warnings
