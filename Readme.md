# QP Compare

This repository keeps a part of apollo-router executing the legacy query planner and comparing its result against that of the new native query planner, which is often called the "semantic diff".

## Build

```
cargo build
```

## Running as a command-line tool

```
cargo run --schema <SCHEMA> --operation <OPERATION>
```

It runs both the legacy and native query planners and prints the generated (native) query plan. If there is a difference between the two planners, its detail will follow.

Run `cargo run -- --help` for additional options.

## Imported as a library

This git repo can be imported as a library. Its crate name is `qp_compare`.

Cargo.toml
```
qp-compare = { git = "https://github.com/apollographql/qp-compare", branch = "main" }
```
