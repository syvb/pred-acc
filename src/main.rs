use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMarket {
    id: String,
    creator_id: String,
    creator_username: String,
    creator_name: String,
    created_time: i64,
    creator_avatar_url: String,
    close_time: i64,
    question: String,
    url: String,
    total_liquidity: Option<f64>, // missing on some markets
    outcome_type: String,
    mechanism: String,
    volume: f64,
    #[serde(rename = "volume24Hours")]
    volume_24_hours: f64,
    is_resolved: bool,
    resolution: Option<String>,
    resolution_time: Option<i64>,
    last_updated_time: i64,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BetList(Vec<RawBet>);

fn main() {
    let data_path: PathBuf = env::args().nth(1).expect("no path :(").parse().unwrap();
    let markets = data_path.join("markets.json");
    let markets =
        fs::read_to_string(markets).expect("couldn't read markets.json  - try `make dl && make`");
    let markets: MarketsJson = serde_json::from_str(&markets).unwrap();

    let bets_dir = data_path.join("bets");
    let mut bets_files = fs::read_dir(&bets_dir)
        .unwrap()
        .map(|file| {
            file.as_ref()
                .unwrap()
                .file_name()
                .into_string()
                .unwrap()
        })
        .filter(|file| file.ends_with(".json"))
        .collect::<Vec<_>>();
    bets_files.sort();
    for bets_file in bets_files {
        println!("Doing bets file {}", bets_file);
        let bets_file = bets_dir.join(bets_file);
        let bets = fs::read_to_string(bets_file).unwrap();
        let bets: BetList = serde_json::from_str(&bets).unwrap();
        for bet in bets.0 {
            if bet.user_username.as_deref() == Some("loops") {
                println!("{:#?}", bet);
            }
        }
    }
}
