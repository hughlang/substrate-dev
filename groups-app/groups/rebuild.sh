./scripts/build.sh
cargo build --release
./target/release/groups purge-chain --dev
./target/release/groups --dev
