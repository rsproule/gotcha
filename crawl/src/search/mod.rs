use async_trait::async_trait;
use ethers::types::Address;
use ethers::types::TxHash;
use ethers_etherscan::account::NormalTransaction;
use serde::Serialize;
pub mod etherscan;

#[async_trait]
pub trait Crawler {
    async fn get_edges(&self, address: &Address) -> Vec<DirectedEdges>;
}

#[derive(Debug)]
pub struct Edges {
    pub from: Address,
    pub to: Address,
    pub txs: Vec<NormalTransaction>,
}

#[derive(Debug, Serialize, PartialEq, Eq, Hash)]
struct Edge {
    from: Address,
    to: Address,
}

#[derive(Debug, Serialize)]
struct Node {
    address: Address,
    label: String,
}

#[derive(Debug)]
pub enum DirectedEdges {
    Forward(Edges),
    Backward(Edges),
}

#[derive(Debug, Serialize)]
pub struct SimpleEdges {
    pub from: Address,
    pub to: Address,
    pub txs: Vec<TxHash>,
}

impl From<Edges> for SimpleEdges {
    fn from(value: Edges) -> Self {
        SimpleEdges {
            from: value.from,
            to: value.to,
            txs: value
                .txs
                .iter()
                .map(|tx| tx.hash.value().copied().unwrap_or(TxHash::zero()))
                .collect(),
        }
    }
}
