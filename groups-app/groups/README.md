# Groups: A Substrate runtime module for creating and managing small groups

The Groups module is designed for the most common use cases for managing and verifying *small* groups of users.
By itself, it only provides a on-chain storage of group membership for a set of AccountIds. Arguably, this does
not need to be stored on-chain since it is application-specific logic. However, in conjunction with other Substrate
modules, the ability to verify group membership before execution of other app/storage logic is useful to provide
auditable proof that group membership rules are not violated. Examples: multiplayer games, multiparty voting

## Features

### Create a group and edit it.

Any user can create a group up to the configurable limit of `max_groups_per_owner`. In creating a group, the owner can provide a name String as a byte array and define the max size of the group, which is limited by the configurable `max_group_size` value. These limits help prevent excessive state bloat. In a gas-cost environment, the system would probably charge appropriately.

The owner can also edit the `name` and the `max_size` of the group. The name field can be used for either storing a human-readable string or a foreign key value for looking up the corresponding data in another datastore. This also has a configurable length limit called `max_name_size`.

### Group membership

The module provides two kinds add/remove functions. Voluntary (opt-in by the user) and Involuntary (owner can add/remove users).

The group members functionality is barebones and is not meant to hold much application-specific logic.
In some group-membership frameworks, there is a notion of an invite or a request to join. This may be
a future enhancement, but it seems more likely that the state information for this should not be
on-chain. Instead, webapps that use this module should listen for events that can be used to store
state information in another datastore.

### Events

Please see the in-line comments in the `decl_event!` section of the `groups.rs` file to see the events that are triggered when specific functions are called and state changes have occured.

### Future

The following are some ideas for future improvements and enhancements.

* Lock the current group and record the timestamp and group member AccountIds:
  * This would be useful for applications where the final roster of members needs to be immutable and verified.
* Record snapshot of current group membership. Possibly just a hash of all member AccountIds sorted, along with other metadata.
* Clone existing group: Easily copy a group (and optionally for a new owner)
* Demonstrate integration with other/future voting module libs.


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
