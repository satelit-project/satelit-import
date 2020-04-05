#!/usr/bin/env bash
#
# Builds and archives project with it's dependencies

set -euo pipefail

DISTRO="distro"

build_project() {
  echo "--- Building Project" >&2
  mkdir -p "$DISTRO"
  cargo build \
    --target x86_64-unknown-linux-musl \
    --release \
    >&2

  mv "target/x86_64-unknown-linux-musl/release/satelit-import" "$DISTRO"
}

build_diesel() {
  echo "--- Building Diesel CLI" >&2
  RUSTFLAGS="${RUSTFLAGS:-} -A deprecated" \
    cargo install diesel_cli \
    --no-default-features \
    --features postgres \
    --root "$(pwd)/$DISTRO/tools" \
    --target x86_64-unknown-linux-musl \
    --git "https://github.com/satelit-project/diesel" \
    --branch "musl-1.4.x" \
    >&2

  mv "$DISTRO/tools/bin/diesel" "$DISTRO/tools"
  rm -r "$DISTRO/tools/bin"
  rm "$DISTRO/tools/".crates*
}

package() {
  echo "--- Packaging Project" >&2
  mkdir -p "$DISTRO"
  cp -R migrations "$DISTRO"
  cp -R config "$DISTRO"
  cp diesel.toml Cargo.* "$DISTRO"
  cp docker/app/scripts/entry.sh "$DISTRO"

  find "$DISTRO/" -type f -o -type l -o -type d \
    | sed s,^"$DISTRO/",, \
    | tar -czf satelit-import.tar.gz \
    --no-recursion -C "$DISTRO/" -T -

  rm -rf "$DISTRO"
}

main() {
  export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
  export SSL_CERT_DIR=/etc/ssl/certs

  build_project
  build_diesel
  package
}

main "$@"
