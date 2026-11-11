use club_reconcile::{Committee, Query, summarize};
use std::{env, fs, process::ExitCode};

fn demo(directory: &str) -> Result<(), Box<dyn std::error::Error>> {
    let query = Query {
        chain: 31337,
        token: format!("0x{:040x}", 1),
        wallet: format!("0x{:040x}", 2),
        first: 100,
        last: 103,
        anchor: format!("0x{:064x}", 103),
    };
    let complete = summarize(
        &query,
        &fs::read_to_string(format!("{directory}/complete.csv"))?,
    )?;
    let missing = summarize(
        &query,
        &fs::read_to_string(format!("{directory}/missing.csv"))?,
    )?;
    let reordered = summarize(
        &query,
        &fs::read_to_string(format!("{directory}/reordered.csv"))?,
    )?;
    let expected = format!("{}\n0x{:040x},45,2\n0x{:040x},70,1", query.key()?, 3, 4);
    assert_eq!(complete, expected);
    assert_eq!(reordered, expected);
    assert_ne!(missing, expected);

    println!("Club reimbursement reconciliation | fixture-based simulation");
    println!("Member A (complete):\n{complete}");
    println!("Member B (one missing transfer):\n{missing}");
    let mut committee = Committee::default();
    committee.vote(0, complete)?;
    committee.vote(1, missing)?;
    assert_eq!(committee.published(), None);
    println!("A + B: PENDING (1 vote per result)");
    committee.vote(2, reordered)?;
    assert_eq!(committee.published(), Some(expected.as_str()));
    println!("A + B + C: PUBLISHED (2 matching votes)\n{expected}");
    assert!(
        committee
            .vote(2, String::from("conflicting retry"))
            .is_err()
    );
    assert!(committee.vote(3, expected).is_err());
    println!("Conflicting retry and unknown member: rejected");
    Ok(())
}

fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args {
        [command, directory] if command == "demo" => demo(directory),
        [command, chain, token, wallet, first, last, anchor, csv] if command == "summarize" => {
            let query = Query {
                chain: chain.parse()?,
                token: token.to_ascii_lowercase(),
                wallet: wallet.to_ascii_lowercase(),
                first: first.parse()?,
                last: last.parse()?,
                anchor: anchor.to_ascii_lowercase(),
            };
            println!("{}", summarize(&query, &fs::read_to_string(csv)?)?);
            Ok(())
        }
        _ => Err("Usage: club-reconcile demo <fixture-directory>\n       club-reconcile summarize <chain> <token> <wallet> <first> <last> <anchor> <csv>".into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
