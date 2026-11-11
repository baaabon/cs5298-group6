# Club Reconcile

## Problem and use case

A student club treasurer reviewing reimbursements needs to reconcile outgoing token transfers with a payment register. A single export can silently omit transfers and produce an understated total. Club Reconcile summarizes outgoing transfers by recipient for a fixed query and publishes a snapshot when two of three configured accounts submit matching results.

## Design and implementation

The Rust core filters events by token, treasury wallet and inclusive block range. It checks the export header against the configured chain and end-block hash, rejects duplicate transaction-hash and log-index pairs, and sums raw token amounts with checked arithmetic. The snapshot starts with the query and lists payees in address order, making identical results comparable even when exports arrive in different orders.

The Lyquid instance method computes a snapshot from CSV supplied by the caller. Network state stores the configured signer addresses, one vote per signer, and the published snapshot. Transactions are authenticated by caller address. An identical retry does not add another vote, and a conflicting retry is rejected. The deployment uses a fixed synthetic query.

## Evaluation

Run the Rust tests and the CLI demo with the README commands. For the local devnet run, use LDK 0.8.4 and three funded Anvil test accounts. The test waits until the node has processed two submissions before checking that the published result is empty. It then submits the third vote and checks for an exact match with the complete snapshot.

| Scenario | Observed result |
|---|---|
| Complete export | Recipient 0x…03 received 45 units in two payments. Recipient 0x…04 received 70 units in one payment. |
| Export missing a 25-unit payment | Recipient 0x…03 was undercounted at 20 units in one payment. |
| Two differing votes on Lyquor | PENDING. No snapshot published. |
| Third vote from a complete export in reverse order | PUBLISHED. The published totals were 45 and 70, with payment counts of two and one. |
| Rust regression tests | Three passed, covering input order, wrong anchor, duplicate events and repeated or conflicting votes. |

Evidence: evidence/devnet-run.txt, evidence/cli-run.txt and evidence/rust-tests.txt. The observed deployment used one local Lyquor node and three test signer accounts. The deployed Lyquid ID was Lyquid-wsd6v4xjliqgjrcaiikuafrjmaho4q4hqiwqa.

## Limitations and next steps

The fixture transfers and anchor are synthetic. Matching votes show agreement under the committee policy. If two signers collude or receive the same incomplete feed, the policy can publish a wrong result. The current implementation relies on signers to submit the result of the instance computation and does not reconcile invoices or verify source-log completeness. A follow-up evaluation could compare independently operated RPC feeds and check block finality against each feed before accepting an export.

## Contributions and AI use

Codex assisted with the Rust core, Lyquid adapter, fixtures, scripts, deployment diagnosis and report drafting. Windows Rust tests and the Ubuntu WSL 2 localnet run supplied the results above. Add actual team member contributions, human review, external user feedback and the public repository URL before submission.