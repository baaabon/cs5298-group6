$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
cargo run --locked --offline --manifest-path (Join-Path $root 'Cargo.toml') -- demo (Join-Path $root 'fixtures')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
