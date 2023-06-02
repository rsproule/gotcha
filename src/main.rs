use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;
use clap::Parser;
use ethers::prelude::*;
use ethers_etherscan::{account::NormalTransaction, contract::ContractMetadata, Client};
use tokio::sync::Mutex;
mod metadock_client;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    address: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let address: Address = args.address.parse()?;
    println!("Running analysis for address: {}", address.to_string());
    let etherscan_client = Client::new_from_env(Chain::Mainnet)?;

    let counter_party_graph = Arc::new(Mutex::new(CounterPartyGraph::new()));
    get_counter_parties(address, &etherscan_client, Arc::clone(&counter_party_graph)).await?;
    // println!("Counter party data: {:?}", counter_party_graph.lock().await);
    let graph = counter_party_graph.lock().await;
    for node in graph.nodes.values() {
        println!("Node: {:?}", node);
    }
    Ok(())
}

struct CounterPartyGraph {
    nodes: HashMap<Address, CounterParty>,
    edges: HashMap<String, (Address, Address)>,
}

impl CounterPartyGraph {
    fn new() -> Self {
        CounterPartyGraph {
            nodes: HashMap::<Address, CounterParty>::new(),
            edges: HashMap::<String, (Address, Address)>::new(),
        }
    }

    fn contains(&self, address: &Address) -> bool {
        self.nodes.contains_key(address)
    }

    fn add_node(
        &mut self,
        address: Address,
        label: Option<String>,
        contract: Option<ContractMetadata>,
        transactions: Vec<NormalTransaction>,
    ) {
        self.nodes.insert(
            address,
            CounterParty {
                address,
                label,
                transactions,
                contract,
            },
        );
    }

    fn add_edge(&mut self, from: Address, to: Address) {
        let first = if from < to { from } else { to };
        let second = if from < to { to } else { from };
        let key = format!("{}-{}", first, second);
        self.edges.insert(key, (first, second));
    }
}

#[async_recursion]
async fn get_counter_parties(
    address: Address,
    etherscan_client: &Client,
    graph: Arc<Mutex<CounterPartyGraph>>,
) -> anyhow::Result<()> {
    let transactions = etherscan_client.get_transactions(&address, None).await?;
    let mut graph_unlocked = graph.lock().await;

    if graph_unlocked.contains(&address) {
        println!("Seen this address before, returning");
        return Ok(());
    }

    // check whats the deal with this address. Need to stop if it is a smart contract or an exchange
    let binding = etherscan_client.contract_source_code(address).await;
    let metadata = match binding {
        Ok(b) => Some(b),
        Err(_) => None,
    };
    let label = metadock_client::get_address_label(address).await?;
    graph_unlocked.add_node(
        address,
        label.clone(),
        metadata.clone(),
        transactions.clone(),
    );
    if label.is_some() || (metadata.is_some() && metadata.unwrap().items.get(0).is_some()) {
        println!("Found a label for this address, returning");
        println!("Label: {:?}", label);
        return Ok(());
    }

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
        .map(|tx| tx.to.unwrap())
        .collect();

    println!("Received from: {:?}", rec_from.len());
    println!("Sent to: {:?}", sent_to.len());
    for back_address in rec_from {
        graph_unlocked.add_edge(back_address, address);
        get_counter_parties(back_address, etherscan_client, graph.clone()).await?;
    }
    for forward_address in sent_to {
        graph_unlocked.add_edge(forward_address, address);
        get_counter_parties(forward_address, etherscan_client, graph.clone()).await?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct CounterParty {
    address: Address,
    label: Option<String>,
    transactions: Vec<NormalTransaction>,
    contract: Option<ContractMetadata>,
}
