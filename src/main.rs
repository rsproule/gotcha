use std::{collections::HashMap, sync::Arc};

use async_recursion::async_recursion;
use clap::Parser;
use ethers::prelude::*;
use ethers_etherscan::{account::NormalTransaction, contract::ContractMetadata, Client};
use tokio::sync::Mutex;
mod metadock_client;

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
    // todo: move to minimal shared state, need this for exit condition
    let seen = Arc::new(Mutex::new(Vec::<Address>::new()));
    let binding = metadock_client::get_address_label(vec![address]).await?;
    // let label: Option<String> = binding.get(0).and_then(|(_, l)| l.clone());
    get_counter_parties(address, &None, &etherscan_client, Arc::clone(&seen)).await?;
    Ok(())
}

#[async_recursion]
async fn get_counter_parties(
    address: Address,
    label: &Option<String>,
    etherscan_client: &Client,
    seen: Arc<Mutex<Vec<Address>>>,
) -> anyhow::Result<()> {
    let transactions = etherscan_client.get_transactions(&address, None).await?;
    // check whats the deal with this address. Need to stop if it is a smart contract or an exchange
    let binding = etherscan_client.contract_source_code(address).await;
    let metadata = match binding {
        Ok(b) => Some(b),
        Err(e) => {
            println!("Error getting contract metadata: {:?}", e);
            None
        }
    };

    if label.is_some() || (metadata.is_some() && metadata.unwrap().items.get(0).is_some()) {
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
        .map(|tx| match tx.to {
            Some(to) => to,
            None => {
                println!("No to address");
                H160::zero()
            }
        })
        .collect();

    let mut new_nodes = vec![];
    let mut seen_unlocked = seen.lock().await;
    for back_address in rec_from.clone() {
        println!("Edge: {:?} -> {:?}", back_address, address);
        if !seen_unlocked.contains(&back_address) {
            new_nodes.push(back_address);
            seen_unlocked.push(back_address);
        }
    }
    for forward_address in sent_to.clone() {
        println!("Edge: {:?} -> {:?}", address, forward_address);
        if !seen_unlocked.contains(&forward_address) {
            new_nodes.push(forward_address);
            seen_unlocked.push(forward_address);
        }
    }
    drop(seen_unlocked);
    let labelled_addresses = metadock_client::get_address_label(new_nodes.clone()).await?;
    // println!("labelled_addresses: {:?}", labelled_addresses);
    for (address, (_, label)) in labelled_addresses {
        // gonna use this as main output
        println!(
            "Node: {:?}-{}",
            address,
            match label.clone() {
                Some(l) => l,
                None => "".to_string(),
            }
        );
        // if let Some(address) = address {
        if label.is_none() {
            get_counter_parties(address, &label, etherscan_client, Arc::clone(&seen)).await?;
        }
        // }
    }
    Ok(())
}
