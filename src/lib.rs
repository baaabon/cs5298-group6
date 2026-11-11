use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub chain: u64,
    pub token: String,
    pub wallet: String,
    pub first: u64,
    pub last: u64,
    pub anchor: String,
}

pub fn hex_id(value: &str, length: usize) -> bool {
    value.len() == length + 2
        && value.starts_with("0x")
        && value[2..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

impl Query {
    pub fn key(&self) -> Result<String, &'static str> {
        if self.chain == 0
            || self.first > self.last
            || !hex_id(&self.wallet, 40)
            || !hex_id(&self.token, 40)
            || !hex_id(&self.anchor, 64)
        {
            return Err("invalid query");
        }
        Ok(format!(
            "v1:{}:{}:{}:{}:{}:{}",
            self.chain, self.token, self.wallet, self.first, self.last, self.anchor
        ))
    }
}

/// CSV rows: block,tx_hash,log_index,token,sender,recipient,raw_amount.
/// Fixtures represent one independently read, complete range for this query.
pub fn summarize(query: &Query, csv: &str) -> Result<String, &'static str> {
    let key = query.key()?;
    let mut lines = csv.lines();
    let expected = format!("chain={},anchor={}", query.chain, query.anchor);
    if lines.next() != Some(expected.as_str()) {
        return Err("source chain or anchor mismatch");
    }
    let mut seen = BTreeSet::new();
    let mut totals = BTreeMap::<&str, (u128, u64)>::new();
    for line in lines {
        let fields: Vec<_> = line.split(',').collect();
        let [block, tx, index, token, sender, recipient, amount] = fields.as_slice() else {
            return Err("invalid event columns");
        };
        let block = block.parse::<u64>().map_err(|_| "invalid block")?;
        let index = index.parse::<u64>().map_err(|_| "invalid log index")?;
        let amount = amount.parse::<u128>().map_err(|_| "invalid amount")?;
        if !hex_id(tx, 64) || ![token, sender, recipient].iter().all(|id| hex_id(id, 40)) {
            return Err("invalid event identifier");
        }
        if !seen.insert((*tx, index)) {
            return Err("duplicate event");
        }
        if (query.first..=query.last).contains(&block)
            && *token == query.token
            && *sender == query.wallet
            && sender != recipient
        {
            let total = totals.entry(*recipient).or_default();
            total.0 = total.0.checked_add(amount).ok_or("amount overflow")?;
            total.1 = total.1.checked_add(1).ok_or("count overflow")?;
        }
    }
    let mut snapshot = key;
    for (recipient, (amount, count)) in totals {
        snapshot.push_str(&format!("\n{recipient},{amount},{count}"));
    }
    Ok(snapshot)
}

#[derive(Default)]
pub struct Committee {
    votes: BTreeMap<usize, String>,
}

impl Committee {
    pub fn vote(&mut self, member: usize, snapshot: String) -> Result<(), &'static str> {
        if member >= 3 {
            return Err("unknown member");
        }
        match self.votes.get(&member) {
            Some(previous) if previous == &snapshot => Ok(()),
            Some(_) => Err("member already voted"),
            None => {
                self.votes.insert(member, snapshot);
                Ok(())
            }
        }
    }

    pub fn published(&self) -> Option<&str> {
        self.votes
            .values()
            .find(|candidate| {
                self.votes
                    .values()
                    .filter(|vote| *vote == *candidate)
                    .count()
                    >= 2
            })
            .map(String::as_str)
    }
}
