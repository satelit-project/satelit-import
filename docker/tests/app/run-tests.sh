#!/usr/bin/env bash

set -euxo pipefail

pushd repo

# lint code
scripts/clippy.sh

# lint formatting
scripts/fmt.sh --check

# prepare db
./prepare-db.sh

# run tests
cargo test

popd
