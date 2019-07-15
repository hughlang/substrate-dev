# Pool

This is an experimental module for managing a pooled group of funds and the transaction amounts. Several modules are being developed in this runtime to experiment with using the modules creatively to support features for a game of chance.

The information below is still in the planning stage, so don't take any of it seriously.

## Pool functions

* A Pool is primarily represented by a Balance where amounts are added and removed.
* The pool has an AccountId where the Balance is stored
* The pool may be an aggregation of funds across subpools that do not have their own AccountIds?

## Approve functions

* Creates and stores decision records that represent the hash of a set of Approvals


## Groups functions

This is copied from the Groups prototype SRML, also found in this repo.

## Faucet functions

This is a speculative feature that allows permissioned faucet distributions.

* Allows one AccountId to create a fund that is used by others in a Group.

## Research topics

* Session module
* Treasury module (removes need for Pool module)
* Use "era" for controlling duration of a session/game.
* Inherents wrapper around external oracle proof




================================================================

# Developer Notes

Last tested with `rustc 1.37.0-nightly (24a9bcbb7 2019-07-04)`

## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Install required tools:

```bash
./scripts/init.sh
```

Build the WebAssembly binary:

```bash
./scripts/build.sh
```

Build all native code:

```bash
cargo build
```

Run all build steps and purge the chain:

```bash
./rebuild.sh
```


## Run

You can start a development chain with:

```bash
./launch.sh
```
Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

Additional CLI usage options are available and may be shown by running `cargo run -- --help`.

## Test

Unit tests can be run with:

```bash
./test.sh
```
