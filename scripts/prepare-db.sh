#!/usr/bin/env bash

set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)/..";

# Prints used database name for provided file
# Arguments:
#   $1 - name of test file
db_name() {
  local filename="${1}"

  set +e
  # shellcheck disable=SC2016
  cat < "${filename}" | rg '^\W*crate::connection_pool\("(\w+)"\).*$' -r '$1'
  set -e
}

# Prints PostgreSQL server url without db path
postgres_url() {
  local config="${REPO_DIR}/config/test.toml"

  # shellcheck disable=SC2016
  cat < "${config}" | rg '^url\W+=\W+"(postgres:.+)"\W*$' -r '$1'
}

# Creates or resets db at provided url and runs migrations
prepare_db() {
  local url="${1}"

  diesel database reset \
      --database-url "${url}" \
      1>&2
}

# Looks up all used db names in test files and prepares them
main() {
  local test_files="${REPO_DIR}/tests/*_tests/*.rs"

  local url
  url="$(postgres_url)"

  for tf in ${test_files}; do
    local name
    name="$(db_name "${tf}")"

    if [[ -n "${name}" ]]; then
      echo 1>&2 "Found database: ${name}"
      prepare_db "${url}${name}"
      printf 1>&2 "\n"
    fi
  done
}

main "$@"
