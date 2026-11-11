#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cargo run --locked --offline --manifest-path "$root/Cargo.toml" -- demo "$root/fixtures"
