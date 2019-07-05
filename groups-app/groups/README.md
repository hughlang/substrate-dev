# Groups: A Substrate runtime module for managing small permissioned groups

## Groups

* A user can create a group through the UI by providing the name of the group and the max size. The result is a group_id hash that can be used to invite others to join the group.
* The owner of a group can rename the group whenever they want.
* The owner of a group can delete/remove the group whenever they want. Users who visit the group page or any related pages will see an error message. More specifically, when trying to load a group, the code will return an error message.

## Members

The user interface should provide a way to browse and view groups that can be joined. The listing of groups is not supported through blockchain queries, so an offchain database should be used. When a group is created, the event information should be saved to a database. Given this premise, there is a notion of viewing a group and displaying it.

On a group page, the user will have the following options:

* Join the group (if no approval required)
* Request to join the group
* Leave the group

In addition, the owner of the group will have the ability to:

* Admin accept member
* Admin remove member

## Votes

The whole notion of having Groups recorded on a blockchain doesn't really make sense unless there is something worth recording. Usually, that means keeping permanent record of funds or transactions.


			Use cases TODO:
			– Create group
			– Update group
			– Remove group
			– Request to join
			– Accept member
			– Add member
			– Remove member
			– List members
			– Verify member (groupId, accountId)

# Future

The following are some ideas for future improvements.

* Lock the current group and record the timestamp and group member AccountIds:
  * This would be useful for applications where the final roster of members needs to be immutable and verified.
* Clone existing group: Easily copy a group (and optionally for a new owner)




# Developer Notes

## Building

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
