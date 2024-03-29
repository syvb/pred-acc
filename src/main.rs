use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMarket {
    id: String,
    creator_id: Option<String>, // missing in Issac market dumps
    creator_username: String,
    creator_name: Option<String>, // missing in Issac market dumps
    created_time: i64,
    creator_avatar_url: Option<String>, // missing in Issac market dumps
    close_time: Option<i64>,            // a few are missing in Issac market dumps
    question: String,
    url: Option<String>,          // missing in Issac market dumps
    total_liquidity: Option<f64>, // missing on some markets
    outcome_type: Option<String>, // missing in Issac market dumps
    mechanism: Option<String>,    // missing in Issac market dumps
    #[serde(rename = "type")]
    type_: Option<String>, // only in Issac dumps
    volume: f64,
    #[serde(rename = "volume24Hours")]
    volume_24_hours: f64,
    is_resolved: Option<bool>,
    resolution: Option<serde_json::Value>,
    resolution_time: Option<i64>,
    last_updated_time: i64,
}
impl RawMarket {
    fn resolution_bool(&self, mut answer: Option<&str>) -> Option<bool> {
        if answer == Some("undefined") {
            answer = None;
        };
        let resolution = self.resolution.as_ref()?;
        let resolution = match resolution {
            serde_json::Value::String(s) => &*s,
            // if it's a float then it resolved at MKT
            serde_json::Value::Number(num) => match num.as_i64()? {
                0 => "NO",
                1 => "YES",
                num => panic!("unknown resolution {:?}", num),
            },
            _ => panic!("unknown resolution {:?}", resolution),
        };
        match (resolution, answer) {
            ("YES", None) => Some(true),
            ("NO", None) => Some(false),
            ("CANCEL", _) | ("CHOOSE_MULTIPLE", Some(_)) | ("MKT", None) => None,
            (correct_id, Some(answer)) if correct_id.len() == 12 => Some(correct_id == answer),
            _ => panic!("Unknown resolution ({:?}, {:?})", self.resolution, answer),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MarketsJson(Vec<RawMarket>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawBet {
    id: String,
    // TODO: fees + fills
    amount: f64,
    is_ante: bool,
    shares: f64,
    user_id: String,
    outcome: String,
    is_filled: Option<bool>,
    user_name: Option<String>,
    limit_prob: Option<f64>,
    prob_after: f64,
    contract_id: String,
    loan_amount: Option<f64>, // missing on some bets
    prob_before: f64,
    visibility: String,
    created_time: i64,
    is_cancelled: Option<bool>,
    is_challenge: bool,
    order_amount: Option<f64>,
    is_redemption: bool,
    user_username: Option<String>,
    user_avatar_url: Option<String>,
    answer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BetList(Vec<RawBet>);

fn main() {
    let data_path: PathBuf = env::args().nth(1).expect("no path :(").parse().unwrap();
    let raw_markets_issac = data_path.join("allMarketData.json");
    let raw_markets_issac = fs::read_to_string(raw_markets_issac)
        .expect("couldn't read markets.json  - try `make dl && make`");
    let raw_markets_issac: MarketsJson = serde_json::from_str(&raw_markets_issac).unwrap();

    let raw_markets_dump = data_path.join("markets.json");
    let raw_markets_dump = fs::read_to_string(raw_markets_dump)
        .expect("couldn't read markets.json  - try `make dl && make`");
    let raw_markets_dump: MarketsJson = serde_json::from_str(&raw_markets_dump).unwrap();
    let mut markets = BTreeMap::new();
    for market in raw_markets_issac.0 {
        markets.insert(market.id.clone(), market);
    }
    // dump data is better so perfer that over Issac's data
    for market in raw_markets_dump.0 {
        markets.insert(market.id.clone(), market);
    }
    let markets = markets;

    let dump_bets_dir = data_path.join("bets");
    let recent_bets_dir = data_path.join("recent-bets");
    let mut dump_bets_files = fs::read_dir(&dump_bets_dir)
        .unwrap()
        .map(|file| dump_bets_dir.join(file.as_ref().unwrap().file_name().into_string().unwrap()))
        .filter(|file| file.extension().unwrap().to_str().unwrap() == "json")
        .collect::<Vec<_>>();
    dump_bets_files.sort();
    dump_bets_files.reverse(); // oldest to newest
    let mut recent_bets_files = fs::read_dir(&recent_bets_dir)
        .unwrap()
        .map(|file| recent_bets_dir.join(file.as_ref().unwrap().file_name().into_string().unwrap()))
        .filter(|file| file.extension().unwrap().to_str().unwrap() == "json")
        .collect::<Vec<_>>();
    recent_bets_files.sort();
    let bets_files = [dump_bets_files, recent_bets_files].concat();

    // (predictions where it resolved YES, predictions where it resolved NO)
    let mut buckets = vec![(0., 0.); 101];
    let mut market_probs_at_cutoff = BTreeMap::new();
    let mut missing_markets = BTreeSet::new();
    let mut bets_on_delisted = 0usize;
    let mut bet_count = 0usize;
    for bets_file in bets_files {
        eprintln!("Doing bets file {}", bets_file.display());
        let bets_file = bets_file;
        let bets = fs::read_to_string(bets_file.clone()).unwrap();
        let bets: BetList = serde_json::from_str(&bets).unwrap();
        // ugly hack until data dump is updated
        let iter = if bets_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("0000")
        {
            bets.0.into_iter()
        } else {
            // TODO: avoid collect here
            bets.0.into_iter().rev().collect::<Vec<_>>().into_iter()
        };
        for bet in iter {
            bet_count += 1;
            // if bet.is_redemption || bet.amount <= 0. {
            //     continue;
            // }
            let bucket = (bet.prob_after * 100.0).round() as usize;
            let market = if let Some(market) = markets.get(&bet.contract_id) {
                market
            } else {
                // ignore bets on delisted contracts
                missing_markets.insert(bet.contract_id);
                bets_on_delisted += 1;
                continue;
            };
            if market
                .mechanism
                .as_ref()
                .map(|mech| mech != "cpmm-1")
                .unwrap_or(false)
                || market
                    .type_
                    .as_ref()
                    .map(|t| !t.contains("cpmm-1"))
                    .unwrap_or(false)
                || market.close_time.is_none()
            // issue with some Issac market dumps
            // || ((market.close_time.unwrap() - market.created_time) < (86400000 * 7))
            {
                continue;
            }

            match market.resolution_bool(bet.answer_id.as_deref()) {
                None => continue,
                Some(true) => buckets[bucket].0 += 1.,
                Some(false) => buckets[bucket].1 += 1.,
            };

            const CUTOFF: i64 = 1682913600000;
            if bet.created_time <= CUTOFF && market.close_time.unwrap() >= CUTOFF
            // && market.resolution_time.unwrap() >= CUTOFF
            {
                market_probs_at_cutoff
                    .insert((market.id.clone(), bet.answer_id.clone()), bet.prob_after);
            }
        }
    }

    eprintln!("Processed {} bets", bet_count);
    eprintln!(
        "Ignored {} bets on delisted contracts across {} markets",
        bets_on_delisted,
        missing_markets.len()
    );
    dbg!(missing_markets);
    let mut cutoff_buckets = vec![(0., 0.); 101];
    for ((market_id, answer_id), prob) in market_probs_at_cutoff {
        let market = markets.get(&market_id).unwrap();
        let bucket = (prob * 25.0).round() as usize * 4;
        match market.resolution_bool(answer_id.as_deref()) {
            None => unreachable!(),
            Some(true) => cutoff_buckets[bucket].0 += 1.,
            Some(false) => cutoff_buckets[bucket].1 += 1.,
        };
    }
    print!(
        "{}",
        cutoff_buckets
            .into_iter()
            .enumerate()
            .map(|(idx, (yes, no))| format!("{}\t{}\t{}", idx, yes, no))
            .fold(String::new(), |a, b| a + &b + "\n")
    );
}
