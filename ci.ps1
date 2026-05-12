cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-default-features --features redb
cargo test --workspace --no-default-features --features json
cargo test --workspace --all-features
cargo doc --workspace --all-features --no-deps
cargo test -p rpstate-macros