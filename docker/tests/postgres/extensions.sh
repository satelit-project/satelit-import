#!/usr/bin/env bash

set -euxo pipefail

# enable uuid extension
psql -v ON_ERROR_STOP=1 --username "${POSTGRES_USER}" <<-SQL
create extension if not exists "uuid-ossp";
SQL
