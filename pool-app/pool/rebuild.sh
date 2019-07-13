./scripts/build.sh
cargo build --release
./target/release/pool purge-chain --dev
./target/release/pool --dev
