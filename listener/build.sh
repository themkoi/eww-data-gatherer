#!/bin/bash
set -e

cd "$(dirname "$0")"
cargo clean

# Minimal-size RUSTFLAGS for nightly
export RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none -C target-cpu=native -C link-arg=-s -Zunstable-options"

# Build and install using nightly with build-std and panic_abort
cargo +nightly install --path . --force \
    -Z build-std=std,panic_abort \
    -Z build-std-features=optimize_for_size