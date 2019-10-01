# satelit-import

Orchestrates data import from external soruces.

## Dependencies

**Required:**

- Rust (stable and nightly)
- Docker, Docker Compose and Docker Engine

The app uses PostgreSQL 11 as DB.

See [docker/README.md](./docker/README.md) for more info about Docker.

**Optional:**

- clippy and rustfmt (nightly)
- [diesel_cli](https://github.com/diesel-rs/diesel/tree/master/diesel_cli)
  (`postgres` feature only)
- [ripgrep](https://github.com/BurntSushi/ripgrep)

### Prerequisites

Change your `/etc/hosts` to include following lines:

```
127.0.0.1 import-db
127.0.0.1 import-serve
```

## Test

To execute all unit and integration tests run:

```bash
docker-compose -f docker/tests/docker-compose.yaml up --exit-code-from import-tests # preferred
# ======= or =======
docker-compose -f docker/tests/docker-compose.yaml run --rm import-tests # won't reuse previous containers
```

To run all required services for integration tests and get into test environment without running any tests run:

```bash
docker-compose -f docker/tests/docker-compose.yaml run --rm import-tests bash # for example, to run tests manually
```
