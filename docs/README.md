# ARP Docs

This folder contains the project documentation used by both the mdBook site and the crate-level docs include.

## What Is Here

- narrative docs for contributors and operators,
- the mdBook scaffold under `docs/`,
- generated static output under `docs/book/` after a build,
- source content that is reused by `cargo doc` through `src/lib.rs`.

## Build Code

```sh
cargo build
```

## Build Rust API Docs

```sh
cargo doc --no-deps
```

Generated API docs are written to `target/doc/`.

## Build Book Docs

Install mdBook once if needed:

```sh
cargo install mdbook
```

Build the book:

```sh
mdbook build docs
```

Serve it locally:

```sh
mdbook serve docs -n 127.0.0.1 -p 3001
```