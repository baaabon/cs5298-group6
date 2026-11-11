use club_reconcile::{Committee, Query, summarize};
use lyquid::prelude::*;

state! {
    network member_a: Address = Address::ZERO;
    network member_b: Address = Address::ZERO;
    network member_c: Address = Address::ZERO;
    network vote_a: String = String::new();
    network vote_b: String = String::new();
    network vote_c: String = String::new();
    network published: String = String::new();
}

fn demo_query() -> Query {
    Query {
        chain: 31337,
        token: format!("0x{:040x}", 1),
        wallet: format!("0x{:040x}", 2),
        first: 100,
        last: 103,
        anchor: format!("0x{:064x}", 103),
    }
}

#[method::network(export = eth)]
fn constructor(ctx: &mut _, a: String, b: String, c: String) {
    let a = a.parse::<Address>().expect("invalid member A address");
    let b = b.parse::<Address>().expect("invalid member B address");
    let c = c.parse::<Address>().expect("invalid member C address");
    assert!(
        ![a, b, c].contains(&Address::ZERO) && a != b && a != c && b != c,
        "expected three distinct nonzero accounts"
    );
    *ctx.network.member_a = a;
    *ctx.network.member_b = b;
    *ctx.network.member_c = c;
}

#[method::instance(export = eth)]
fn compute(_ctx: &_, csv: String) -> LyquidResult<String> {
    summarize(&demo_query(), &csv).map_err(|error| LyquidError::LyquorRuntime(error.into()))
}

#[method::network(export = eth)]
fn submit(ctx: &mut _, snapshot: String) -> LyquidResult<bool> {
    let members = [
        *ctx.network.member_a,
        *ctx.network.member_b,
        *ctx.network.member_c,
    ];
    let Some(member) = members.iter().position(|address| *address == ctx.caller) else {
        return Err(LyquidError::LyquorRuntime("unknown member".into()));
    };
    let key = demo_query().key().expect("valid demo query");
    if snapshot.len() > 8192 || snapshot.lines().next() != Some(key.as_str()) {
        return Err(LyquidError::LyquorRuntime(
            "invalid snapshot envelope".into(),
        ));
    }
    let mut committee = Committee::default();
    for (index, previous) in [
        ctx.network.vote_a.as_str(),
        ctx.network.vote_b.as_str(),
        ctx.network.vote_c.as_str(),
    ]
    .into_iter()
    .enumerate()
    {
        if !previous.is_empty() {
            committee
                .vote(index, previous.to_owned())
                .expect("distinct stored votes");
        }
    }
    committee
        .vote(member, snapshot.clone())
        .map_err(|error| LyquidError::LyquorRuntime(error.into()))?;
    match member {
        0 => *ctx.network.vote_a = snapshot,
        1 => *ctx.network.vote_b = snapshot,
        _ => *ctx.network.vote_c = snapshot,
    }
    if let Some(result) = committee.published() {
        *ctx.network.published = result.to_owned();
    }
    Ok(true)
}

#[method::network(export = eth)]
fn result(ctx: &_) -> LyquidResult<String> {
    Ok(ctx.network.published.to_string())
}

#[method::network(export = eth)]
fn vote_count(ctx: &_) -> LyquidResult<u64> {
    Ok([
        ctx.network.vote_a.is_empty(),
        ctx.network.vote_b.is_empty(),
        ctx.network.vote_c.is_empty(),
    ]
    .into_iter()
    .filter(|empty| !*empty)
    .count() as u64)
}
