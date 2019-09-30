#!/usr/bin/env bash

set -euxo pipefail

ARCH="$(uname -i)"
RUST_NIGHTLY_TARGET="${ARCH}-unknown-linux-gnu"
RUST_COMPONENTS_URL="https://rust-lang.github.io/rustup-components-history"

# install required packages
apt-get update && apt-get -yq install curl build-essential pkg-config libssl-dev libpq-dev
apt-get clean && rm -rf /var/lib/apt/lists

# nightly toolchain name
nightly_date=$(curl "${RUST_COMPONENTS_URL}/${RUST_NIGHTLY_TARGET}/rustfmt")
nightly_name="nightly-${nightly_date}-${ARCH}"

# install rustup, toolchain and components
curl https://sh.rustup.rs -sSf | bash -s -- -y --no-modify-path --default-toolchain none
rustup toolchain install stable
rustup component add clippy --toolchain stable

rustup toolchain install "${nightly_name}"
rustup component add rustfmt --toolchain "${nightly_name}"

rustup default stable

# install tools 
cargo install diesel_cli --force --no-default-features --features "postgres"
cargo install ripgrep --force

# check installation
cargo --version
cargo clippy --version
cargo "+${nightly_name}" fmt --version
diesel --version
rg --version
