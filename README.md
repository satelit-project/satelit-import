# satelit-import

Orchestrates data import from external soruces.

## Dependencies

**Required:**

- Rust (stable and nightly)
- Docker and Docker Compose

The app uses PostgreSQL 11 as DB.

See [docker/README.md](./docker/README.md) for more info about Docker.

**Optional:**

- clippy and rustfmt (nightly)
- [diesel_cli](https://github.com/diesel-rs/diesel/tree/master/diesel_cli)
  (`postgres` feature only)
- [ripgrep](https://github.com/BurntSushi/ripgrep)
