use std::collections::HashMap;

use async_trait::async_trait;
use ethers::types::{Address, Chain};
use ethers_etherscan::{account::NormalTransaction, Client};

use crate::search::Edges;

use super::{Crawler, DirectedEdges, Edge};

pub struct EtherscanCrawler {
    client: Client,
}

impl Default for EtherscanCrawler {
    fn default() -> Self {
        EtherscanCrawler {
            client: Client::new_from_env(Chain::Mainnet)
                .expect("could not create etherscan client"),
        }
    }
}

#[async_trait]
impl Crawler for EtherscanCrawler {
    async fn get_edges(&self, address: &Address) -> Vec<DirectedEdges> {
        let transactions = self.client.get_transactions(address, None).await;
        match transactions {
            Ok(t) => get_counter_parties(address, t),
            Err(e) => {
                println!("Etherscan crawler error for {:?}: {:?}", address, e);
                vec![]
            }
        }
    }
}
fn get_counter_parties(
    address: &Address,
    transactions: Vec<NormalTransaction>,
) -> Vec<DirectedEdges> {
    let mapped = transactions
        .iter()
        .map(|tx| {
            let binding = Address::zero();
            let from: &Address = tx.from.value().unwrap_or(&binding);
            let to: Address = tx
                .to
                // fallback to contract address. may be additional modes
                .unwrap_or(tx.contract_address.unwrap_or(Address::zero()));
            (Edge { from: *from, to }, tx)
        })
        .fold(HashMap::new(), |mut acc, edge| {
            let (key, val) = edge;
            acc.entry(key).or_insert_with(Vec::new).push(val.clone());
            acc
        });

    let mut result = Vec::new();
    for (edge, txs) in mapped {
        let de = if edge.from == *address {
            DirectedEdges::Forward(Edges {
                from: edge.from,
                to: edge.to,
                txs: txs.to_vec(),
            })
        } else {
            DirectedEdges::Backward(Edges {
                from: edge.from,
                to: edge.to,
                txs: txs.to_vec(),
            })
        };
        result.push(de);
    }
    result
}
