# satelit-import

## Dependencies

- Rust (latest stable and nightly)
- Clippy and Rustfmt
- [diesel_cli](https://github.com/diesel-rs/diesel/tree/master/diesel_cli)
  (`postgres` feature only)
- [ripgrep](https://github.com/BurntSushi/ripgrep)
- PostgreSQL 11

## TODO:

- ci with checks from `scripts` dir
- unit and integration tests

## TODO in satelit-scheduler:

- Daily job to finish 'dead' import tasks 
- once per month do full db reimport to cleanup dead and reimport lost items
