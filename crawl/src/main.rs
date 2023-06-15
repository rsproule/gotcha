use std::sync::Arc;

use async_recursion::async_recursion;
use clap::Parser;
use ethers::prelude::*;
use search::{Crawler, DirectedEdges};
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{
    label::cache::LabelCache,
    search::{etherscan::EtherscanCrawler, SimpleEdges},
};
mod label;
mod search;

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
    forward: Option<bool>,

    #[arg(long)]
    backward: Option<bool>,

    #[arg(short, long)]
    mode: Option<SearchMode>,
}

#[derive(Debug, PartialEq, Clone)]
struct SearchSettings {
    fan_out_limit: usize,
    recursive_depth: u32,
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
    walk(
        address,
        &etherscan_crawler,
        &label_cache_client,
        Arc::clone(&seen),
        0,
        &SearchSettings {
            fan_out_limit: args.fan_out_limit.unwrap_or(FAN_OUT_LIMIT),
            recursive_depth: args.recursive_depth.unwrap_or(2),
            forward: args.forward.unwrap_or(true),
            backward: args.backward.unwrap_or(true),
            mode: args.mode.unwrap_or(SearchMode::DepthFirst),
        },
    )
    .await?;
    println!("Done!");
    Ok(())
}

#[async_recursion]
async fn walk(
    address: Address,
    etherscan_crawler: &EtherscanCrawler,
    label_cache_client: &LabelCache,
    seen: Arc<Mutex<Vec<Address>>>,
    mut current_depth: u32,
    settings: &SearchSettings,
) -> anyhow::Result<()> {
    current_depth += 1;
    if current_depth > settings.recursive_depth {
        println!("Reached max depth");
        return Ok(());
    }
    let neighbor_edges = etherscan_crawler.get_edges(&address).await;
    let mut new_nodes = vec![];
    let mut seen_unlocked = seen.lock().await;
    for edge in neighbor_edges {
        match edge {
            DirectedEdges::Forward(edge) => {
                if settings.forward && !seen_unlocked.contains(&edge.to) {
                    new_nodes.push(edge.to);
                    seen_unlocked.push(edge.to);
                }
                let simple_edge: SimpleEdges = edge.into();
                println!("Edge:{}", serde_json::to_string(&simple_edge)?);
            }
            DirectedEdges::Backward(edge) => {
                if settings.backward && !seen_unlocked.contains(&edge.from) {
                    new_nodes.push(edge.from);
                    seen_unlocked.push(edge.from);
                }
                let simple_edge: SimpleEdges = edge.into();
                println!("Edge:{}", serde_json::to_string(&simple_edge)?);
            }
        }
    }
    drop(seen_unlocked);
    let labelled_addresses = label_cache_client.get_labels(new_nodes.clone()).await;
    for node in new_nodes {
        let label = labelled_addresses.get(&node);
        match label {
            Some(label) => println!(
                "Node: id=[{:?}] label=[{}] depth=[{}]",
                node, label, current_depth
            ),
            None => {
                println!(
                    "Node: id=[{:?}] label=[UNLABELLED] depth=[{}]",
                    node, current_depth
                );
                walk(
                    node,
                    etherscan_crawler,
                    label_cache_client,
                    Arc::clone(&seen),
                    current_depth,
                    settings,
                )
                .await?;
            }
        }
    }
    Ok(())
}
