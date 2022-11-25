#!/bin/bash
set -e

cargo test
cargo test --examples
$(dirname "$0")/clippy-pedantic.sh
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo bench --no-run --profile dev