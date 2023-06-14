use std::sync::Arc;

use async_recursion::async_recursion;
use clap::Parser;
use ethers::prelude::*;
use ethers_etherscan::{account::NormalTransaction, Client};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::label::cache::LabelCache;
mod label;

const FAN_OUT_LIMIT: usize = 500;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    address: String,

    #[arg(short, long)]
    fan_out_limit: Option<usize>,

    #[arg(short, long)]
    recursive_depth: Option<u32>,

    #[arg(long)]
    forward_only: Option<bool>,

    #[arg(long)]
    backward_only: Option<bool>,

    #[arg(short, long)]
    mode: Option<SearchMode>,
}

#[derive(Debug, PartialEq, Clone)]
struct SearchSettings {
    fan_out_limit: usize,
    recursive_depth: u32,
    forward_only: bool,
    backward_only: bool,
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
    let etherscan_client = Client::new_from_env(Chain::Mainnet)?;
    let label_cache_client = LabelCache::new();
    let seen = Arc::new(Mutex::new(Vec::<Address>::new()));
    println!("Node: id=[{:?}] label=[{}]", address, "STARTER");
    walk(
        address,
        &etherscan_client,
        &label_cache_client,
        Arc::clone(&seen),
        0,
        &SearchSettings {
            fan_out_limit: args.fan_out_limit.unwrap_or(FAN_OUT_LIMIT),
            recursive_depth: args.recursive_depth.unwrap_or(5),
            forward_only: args.forward_only.unwrap_or(false),
            backward_only: args.backward_only.unwrap_or(false),
            mode: args.mode.unwrap_or(SearchMode::DepthFirst),
        },
    )
    .await?;
    Ok(())
}

#[async_recursion]
async fn walk(
    address: Address,
    etherscan_client: &Client,
    label_cache_client: &LabelCache,
    seen: Arc<Mutex<Vec<Address>>>,
    current_depth: u32,
    settings: &SearchSettings,
) -> anyhow::Result<()> {
    if current_depth > settings.recursive_depth {
        println!("Reached max depth");
        return Ok(());
    }
    // cache this?
    let transactions = etherscan_client.get_transactions(&address, None).await?;
    let (rec_from, sent_to) = get_counter_parties(address, &transactions);
    let mut new_nodes = vec![];
    let mut seen_unlocked = seen.lock().await;
    if !settings.forward_only {
        for back_address in rec_from.clone() {
            println!("Edge: {:?} -> {:?}", back_address, address);
            if !seen_unlocked.contains(&back_address) {
                new_nodes.push(back_address);
                seen_unlocked.push(back_address);
            }
        }
    }
    if !settings.backward_only {
        for forward_address in sent_to.clone() {
            println!("Edge: {:?} -> {:?}", address, forward_address);
            if !seen_unlocked.contains(&forward_address) {
                new_nodes.push(forward_address);
                seen_unlocked.push(forward_address);
            }
        }
    }
    drop(seen_unlocked);
    let labelled_addresses = label_cache_client.get_labels(new_nodes.clone()).await;
    for node in new_nodes {
        let label = labelled_addresses.get(&node);
        match label {
            Some(label) => println!("Node: id=[{:?}] label=[{}]", node, label),
            None => {
                println!("Node: id=[{:?}] label=[UNLABELLED]", node);
                walk(
                    node,
                    etherscan_client,
                    label_cache_client,
                    Arc::clone(&seen),
                    current_depth + 1,
                    settings,
                )
                .await?;
            }
        }
    }
    Ok(())
}

fn get_counter_parties(
    address: Address,
    transactions: &[NormalTransaction],
) -> (Vec<Address>, Vec<Address>) {
    let rec_from: Vec<H160> = transactions
        .iter()
        .filter(|tx| match tx.to {
            Some(to) => to == address,
            None => false,
        })
        .map(|tx| *tx.from.value().unwrap())
        .collect();

    let sent_to: Vec<Address> = transactions
        .iter()
        .filter(|tx| match tx.from.value() {
            Some(from) => from == &address,
            None => false,
        })
        .map(|tx| match tx.to {
            Some(to) => to,
            None => {
                println!("No to address");
                H160::zero()
            }
        })
        .collect();
    (rec_from, sent_to)
}

#[derive(Debug, Serialize)]
struct Node {
    address: Address,
    label: String,
}

#[derive(Debug, Serialize)]
struct Edge {
    tx: Transaction,
    from: Address,
    to: Address,
    amount: U256,
}
