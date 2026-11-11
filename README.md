# Club Reconcile

Club Reconcile is a student club reimbursement reconciler for CS5298 Direction 3, Verifiable Indexer. It groups outgoing token transfers by recipient over a historical block range. Three authorized accounts can submit summaries, and the Lyquid publishes a result once two snapshots match exactly.

The verified setup used Ubuntu on WSL 2, a single Lyquor node, LDK 0.8.4, three Anvil test accounts, and synthetic CSV data. The standalone Rust CLI and regression tests also passed on Windows.

## Project structure

| Path | Purpose |
|---|---|
| [src/](src/) | [lib.rs](src/lib.rs) validates CSV data, filters transfers, computes totals and counts, and applies the 2-of-3 voting rule. [main.rs](src/main.rs) provides the CLI. |
| [lyquid/](lyquid/) | [src/lib.rs](lyquid/src/lib.rs) exposes the Rust logic through Lyquor instance and network methods. [Cargo.toml](lyquid/Cargo.toml) provides the template for the temporary deployment manifest. |
| [fixtures/](fixtures/) | Three CSV exports representing complete, incomplete, and reordered data. |
| [scripts/](scripts/) | [e2e.ps1](scripts/e2e.ps1) and [e2e.sh](scripts/e2e.sh) run the CLI demo. [devnet-e2e.sh](scripts/devnet-e2e.sh) builds, deploys, and tests the Lyquid. |
| [tests/](tests/) | [reconciliation.rs](tests/reconciliation.rs) checks input order, invalid anchors, duplicate events, and repeated or conflicting votes. |
| [evidence/](evidence/) | Saved CLI, test, and deployment logs. [ldk-check.txt](evidence/ldk-check.txt) records an earlier Windows dependency download failure. |
| `target/`, `lyquid/target/`, `lyquid_tools_target/` | Generated Cargo and Shaker build output and caches. |
| `lyquid/ldk` | A local SDK link from early debugging, if present. The current deployment script resolves the installed SDK directly. |
| `.git/` | Local Git metadata. |

[Cargo.toml](Cargo.toml) and [Cargo.lock](Cargo.lock) define the standalone Rust project. [REPORT-DRAFT.md](REPORT-DRAFT.md) describes the design, experiment, and limitations.

## 1. Inspect the test data

The demo uses chain ID `31337`, blocks `100–103`, a test token ending in `01`, and a club wallet ending in `02`. The CSV header records the chain ID and end-block hash. Each event contains the block height, transaction hash, log index, token, sender, recipient, and amount in raw token units. CSV addresses and hashes use lowercase hexadecimal with a `0x` prefix.

| Member | Input | Expected result |
|---|---|---|
| A | [complete.csv](fixtures/complete.csv) | Recipient `…03` receives 45 units across two transfers. Recipient `…04` receives 70 units in one transfer. The query filters out a 999-unit transfer from another wallet. |
| B | [missing.csv](fixtures/missing.csv) | This export omits a 25-unit transfer to `…03`, reducing that recipient's total to 20 units in one transfer. The total for `…04` stays at 70. |
| C | [reordered.csv](fixtures/reordered.csv) | The complete export in reverse order. Sorting the recipients produces the same snapshot as A. |

## 2. Run the Rust tests and CLI demo

Use Rust 1.85 or later. The CLI uses only the Rust standard library and runs offline. In Windows PowerShell, enter the repository and run:

```powershell
Set-Location -LiteralPath 'D:\HuaweiMoveData\Users\86137\Documents\ChatGPT\chain p'
cargo test --locked --offline
& .\scripts\e2e.ps1
```

Use your checkout path if it differs. If PowerShell blocks script execution, run `cargo run --locked --offline -- demo fixtures`. In Ubuntu, run `bash scripts/e2e.sh` from the repository.

- **Tests:** All three passed. They cover row order, an invalid anchor, duplicate events, and repeat or conflicting votes. [Rust test log](evidence/rust-tests.txt)
- **CLI demo:** The output shows the difference between A and B, followed by `A + B: PENDING` and `A + B + C: PUBLISHED`. The demo also checks that an unknown member and a conflicting retry are rejected. [CLI run log](evidence/cli-run.txt)

For your own CSV export, run:

```text
cargo run --locked --offline -- summarize <chain> <token> <wallet> <first> <last> <anchor> <csv>
```

Pass the chain ID and inclusive block bounds as decimal integers. Pass the anchor as a 32-byte hexadecimal hash with a `0x` prefix. This command prints one snapshot.

## 3. Start Lyquor in the first Ubuntu terminal

Install the Linux LDK 0.8.4 distribution with Shaker and Foundry. Install jq with `sudo apt update && sudo apt install -y jq` if needed. [Official Lyquor devnet guide](https://docs.lyquor.dev/docs/ldk/build-deploy-test/)

```bash
for tool in cargo shaker cast anvil jq; do command -v "$tool"; done
bash "$HOME/.shakenup/ldk/scripts/localnet.sh" single
```

The script starts Anvil and deploys the `bartender` registry. Wait for `Lyquor localnet (single) is up with 1 node(s).` The node exposes `http://127.0.0.1:10087/api` and `ws://127.0.0.1:10087/ws`. Keep this terminal running while you deploy from a second Ubuntu terminal.

## 4. Deploy and test from the second Ubuntu terminal

Enter the repository. The command derives three accounts from Anvil's public development mnemonic for use on the local devnet. Set `LYQUOR_LDK_DIR` if your SDK source directory differs from `~/.shakenup/ldk`.

```bash
cd '/mnt/d/HuaweiMoveData/Users/86137/Documents/ChatGPT/chain p'

(
  set -euo pipefail
  mnemonic='test test test test test test test test test test test junk'
  SIGNER_A="$(cast wallet private-key --mnemonic "$mnemonic" --mnemonic-index 0)"
  SIGNER_B="$(cast wallet private-key --mnemonic "$mnemonic" --mnemonic-index 1)"
  SIGNER_C="$(cast wallet private-key --mnemonic "$mnemonic" --mnemonic-index 2)"
  export SIGNER_A SIGNER_B SIGNER_C
  bash scripts/devnet-e2e.sh 2>&1 | tee evidence/devnet-rerun.txt
)
```

This writes `evidence/devnet-rerun.txt`. The [original successful run](evidence/devnet-run.txt) remains available for comparison. Contract addresses and Lyquid IDs can differ between deployments. If Shaker reports `Lyquid ID: pending`, the script stops. Check the localnet node's `lyquor.log` for discovery and hosting events.

| Step | What the script does | Verified result |
|---|---|---|
| Build and deploy | Stages a Cargo manifest in a temporary directory and references the installed SDK and shared Rust core. Shaker builds and pushes the image. The constructor parses and stores three distinct nonzero signer addresses. | Shaker reported `Built club-reconcile-lyquid with LDK 0.8.4`, `Status: created`, and a resolved Lyquid ID. [Deployment log](evidence/devnet-run.txt) |
| Compute three snapshots | Calls the `compute(string)` instance method for each CSV. The Rust core checks the chain and anchor, rejects duplicate logs, filters by the query, and sorts recipient totals. | The script verified that A and C match and that B differs before submitting votes. [Devnet run](evidence/devnet-run.txt) · [Individual totals](evidence/cli-run.txt) |
| Submit A and B | Sends one signed `submit(string)` transaction from each account. Waits until `vote_count()` returns two, then reads `result()`. | The result was empty and the script printed `A + B: PENDING`. [Vote check](evidence/devnet-run.txt) |
| Submit C and read the result | Submits C's snapshot and polls `result()` until it matches the full snapshot byte for byte. | The script printed `A + B + C: PUBLISHED`, followed by the query and both recipient totals. [Published result](evidence/devnet-run.txt) |

The successful run ended with:

```text
A + B: PENDING
A + B + C: PUBLISHED
v1:31337:0x0000000000000000000000000000000000000001:0x0000000000000000000000000000000000000002:100:103:0x0000000000000000000000000000000000000000000000000000000000000067
0x0000000000000000000000000000000000000003,45,2
0x0000000000000000000000000000000000000004,70,1
```

The first line of the snapshot encodes `version:chain ID:token:sender wallet:first block:last block:anchor`. Each subsequent line contains `recipient,amount,transfer count`.

## Verification scope

The single-node experiment verified deployment, instance computation, authenticated voting by three test accounts, and publication after two matching votes. The transfers and block anchor were synthetic. The protocol relies on authorized accounts to submit computed results. Two signers using the same incomplete export can publish an incorrect summary. Live log ingestion, block finality verification, and invoice matching are outside the current implementation.
