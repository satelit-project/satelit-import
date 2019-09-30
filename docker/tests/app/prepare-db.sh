#!/usr/bin/env bash
# Looks up all used database names in tests and prepares those databases
#
# Usage: ./prepare-db.sh

set -euo pipefail

if [[ -d ".git" ]]; then
  REPO_DIR="$(git rev-parse --show-toplevel)"
else
  REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)/repo";
fi

# Prints used database name for provided file
# Arguments:
#   $1 - name of test file
db_name() {
  local filename="${1}"

  set +e
  # shellcheck disable=SC2016
  cat < "${filename}" | rg '^\W*crate::connection_pool\("([\w\-_]+)"\).*$' -r '$1'
  set -e
}

# Prints PostgreSQL server url without db path
# Arguments:
#   $1 - config name without extension
postgres_url() {
  local config="${REPO_DIR}/config/${1}.toml"

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

main() {
  local suites=( "db_tests" "rpc_tests" )
  pushd "${REPO_DIR}" >/dev/null

  for suite in "${suites[@]}"; do
    local url
    url=$(postgres_url "${suite}")

    local test_files="${REPO_DIR}/tests/${suite}/*.rs"

    for tf in ${test_files}; do
      local name
      name="$(db_name "${tf}")"

      if [[ -n "${name}" ]]; then
        echo 1>&2 "Found database: ${name}"
        prepare_db "${url}${name}"
        printf 1>&2 "\n"
      fi
    done
  done

  popd >/dev/null
}

main "$@"
