use std::sync::Arc;

use async_recursion::async_recursion;
use ethers::types::Address;
use tokio::sync::Mutex;

use crate::{search::{etherscan::EtherscanCrawler, Crawler, DirectedEdges, SimpleEdges}, label::cache::LabelCache, SearchSettings};


#[async_recursion]
pub async fn crawl(
    address: Address,
    etherscan_crawler: &EtherscanCrawler,
    label_cache_client: &LabelCache,
    seen: Arc<Mutex<Vec<Address>>>,
    mut current_depth: usize,
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
                crawl(
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