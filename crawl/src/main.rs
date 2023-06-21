use std::sync::Arc;

use clap::Parser;
use ethers::prelude::*;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{label::cache::LabelCache, search::etherscan::EtherscanCrawler, crawler::crawl};
mod crawler;
mod label;
mod search;

const FAN_OUT_LIMIT: usize = 50;
const RECURSIVE_DEPTH: usize = 3;
const ENABLE_FORWARD: bool = true;
const ENABLE_BACKWARD: bool = true;
const DEFAULT_MODE: SearchMode = SearchMode::DepthFirst;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    address: String,

    #[arg(short, long)]
    fan_out_limit: Option<usize>,

    #[arg(short, long)]
    recursive_depth: Option<usize>,

    #[arg(long)]
    forward: Option<bool>,

    #[arg(long)]
    backward: Option<bool>,

    #[arg(short, long)]
    mode: Option<SearchMode>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SearchSettings {
    fan_out_limit: usize,
    recursive_depth: usize,
    forward: bool,
    backward: bool,
    mode: SearchMode,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
enum SearchMode {
    DepthFirst,
    BreadthFirst,
}

impl From<String> for SearchMode {
    fn from(value: String) -> Self {
        match value.as_str() {
            "depth-first" => SearchMode::DepthFirst,
            "dfs" => SearchMode::DepthFirst,
            "breadth-first" => SearchMode::BreadthFirst,
            "bfs" => SearchMode::BreadthFirst,
            _ => SearchMode::DepthFirst,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let address: Address = args.address.parse()?;
    let label_cache_client = LabelCache::new();
    let etherscan_crawler = EtherscanCrawler::default();
    let seen = Arc::new(Mutex::new(Vec::<Address>::new()));
    let label = label_cache_client.get_label(address).await;
    println!(
        "Node: id=[{:?}] label=[{}:{}] depth=[0]",
        address,
        "STARTER",
        label.unwrap_or("NO-LABEL".to_string())
    );
    let mut l = seen.lock().await;
    l.push(address);
    drop(l);
    crawl(
        address,
        &etherscan_crawler,
        &label_cache_client,
        Arc::clone(&seen),
        0,
        &SearchSettings {
            fan_out_limit: args.fan_out_limit.unwrap_or(FAN_OUT_LIMIT),
            recursive_depth: args.recursive_depth.unwrap_or(RECURSIVE_DEPTH),
            forward: args.forward.unwrap_or(ENABLE_FORWARD),
            backward: args.backward.unwrap_or(ENABLE_BACKWARD),
            mode: args.mode.unwrap_or(DEFAULT_MODE),
        },
    )
    .await?;
    println!("Done!");
    Ok(())
}
