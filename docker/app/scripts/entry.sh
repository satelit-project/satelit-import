#!/usr/bin/env ash

set -euo pipefail

wait_db() {
  local retries=5
  while [[ "$retries" -gt "0" ]]; do
    set +e
    tools/diesel migration list \
      --database-url "$PG_DB_URL" \
      >&2
    local status="$?"
    set -e

    if [[ "$status" -eq "0" ]]; then
      echo "Database available." >&2
      return
    fi

    retries=$(( retries - 1 ))
    echo "Database is not available. Sleeping..." >&2
    sleep 10s
  done

  exit 1
}

main() {
  echo "Waiting for DB" >&2
  wait_db

  echo "Running migrations" >&2
  tools/diesel setup \
    --database-url "$PG_DB_URL" \
    >&2
  tools/diesel migration run \
    --database-url "$PG_DB_URL" \
    >&2

  echo "Running service" >&2
  ST_LOG=prod \
    DO_SPACES_KEY="$DO_SPACES_KEY" \
    DO_SPACES_SECRET="$DO_SPACES_SECRET" \
    DO_SPACES_HOST="$DO_SPACES_HOST" \
    DO_BUCKET="$DO_BUCKET" \
    DO_REGION="$DO_REGION" \
    PG_DB_URL="$PG_DB_URL" \
    exec ./satelit-import
}

main "$@"
