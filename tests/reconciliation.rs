use club_reconcile::{Committee, Query, summarize};

fn query() -> Query {
    Query {
        chain: 31337,
        token: format!("0x{:040x}", 1),
        wallet: format!("0x{:040x}", 2),
        first: 100,
        last: 103,
        anchor: format!("0x{:064x}", 103),
    }
}

const COMPLETE: &str = include_str!("../fixtures/complete.csv");

#[test]
fn same_transfers_in_different_order_produce_same_snapshot() {
    assert_eq!(
        summarize(&query(), COMPLETE),
        summarize(&query(), include_str!("../fixtures/reordered.csv"))
    );
}

#[test]
fn rejects_wrong_anchor_and_duplicate_event() {
    let wrong_anchor = COMPLETE.replacen("0067", "0068", 1);
    assert_eq!(
        summarize(&query(), &wrong_anchor),
        Err("source chain or anchor mismatch")
    );
    let first_event = COMPLETE.lines().nth(1).unwrap();
    let duplicate = format!("{COMPLETE}{first_event}\n");
    assert_eq!(summarize(&query(), &duplicate), Err("duplicate event"));
}

#[test]
fn duplicate_vote_cannot_create_quorum() {
    let mut committee = Committee::default();
    committee.vote(0, "agreed".into()).unwrap();
    committee.vote(0, "agreed".into()).unwrap();
    assert_eq!(committee.published(), None);
    assert!(committee.vote(0, "changed".into()).is_err());
    committee.vote(1, "agreed".into()).unwrap();
    assert_eq!(committee.published(), Some("agreed"));
    committee.vote(2, "different".into()).unwrap();
    assert_eq!(committee.published(), Some("agreed"));
}
