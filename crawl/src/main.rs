use std::sync::Arc;

use async_recursion::async_recursion;
use clap::Parser;
use ethers::prelude::*;
use ethers_etherscan::{account::NormalTransaction, Client};
use tokio::sync::Mutex;
mod metadock_client;
mod label_client;

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let address: Address = args.address.parse()?;
    let max_depth = args.recursive_depth.unwrap_or(5);
    println!("Running analysis for address: {}", address.to_string());
    let etherscan_client = Client::new_from_env(Chain::Mainnet)?;
    let seen = Arc::new(Mutex::new(Vec::<Address>::new()));
    let local_label = label_client::get_address_labels(vec![address]).await?;
    let binding = metadock_client::get_address_label(vec![address], 1).await?;
    let label: Option<String> = binding.get(&address).and_then(|l| l.clone());
    println!("Label: {:?}", label);
    walk(
        address,
        &None,
        &etherscan_client,
        Arc::clone(&seen),
        0,
        max_depth,
        args.forward_only.unwrap_or(false),
        args.backward_only.unwrap_or(false),
    )
    .await?;
    Ok(())
}

#[async_recursion]
async fn walk(
    address: Address,
    label: &Option<String>,
    etherscan_client: &Client,
    seen: Arc<Mutex<Vec<Address>>>,
    current_depth: u32,
    max_depth: u32,
    forward_only: bool,
    backward_only: bool,
) -> anyhow::Result<()> {
    if current_depth > max_depth {
        println!("Reached max depth");
        return Ok(());
    }
    let transactions = etherscan_client
        .get_transactions(&address, None)
        .await
        .map_err(|e| println!("Error getting transactions for {:?}: {:?}", address, e));
    let transactions = match transactions {
        Ok(t) => t,
        Err(_) => return Ok(()),
    };
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

    let (rec_from, sent_to) = get_counter_parties(address, &transactions);

    let mut new_nodes = vec![];
    let mut seen_unlocked = seen.lock().await;
    if !forward_only {
        for back_address in rec_from.clone() {
            println!("Edge: {:?} -> {:?}", back_address, address);
            if !seen_unlocked.contains(&back_address) {
                new_nodes.push(back_address);
                seen_unlocked.push(back_address);
            }
        }
    }
    if !backward_only {
        for forward_address in sent_to.clone() {
            println!("Edge: {:?} -> {:?}", address, forward_address);
            if !seen_unlocked.contains(&forward_address) {
                new_nodes.push(forward_address);
                seen_unlocked.push(forward_address);
            }
        }
    }
    drop(seen_unlocked);
    let local_label = label_client::get_address_labels(new_nodes.clone()).await?;
    let labelled_addresses = metadock_client::get_address_label(new_nodes.clone(), 1).await?;
    for (address, label) in labelled_addresses {
        // TODO: this doesnt log smart contracts
        println!(
            "Node: id=[{:?}] label=[{}]",
            address,
            match label.clone() {
                Some(l) => l,
                None => {
                    if current_depth == 0 {
                        "STARTER".to_string()
                    } else {
                        // this is horribly hacky, need to adjust this so we do the etherscan at same time as metadock
                        let binding = etherscan_client.contract_source_code(address).await;
                        if let Ok(metadata) = binding {
                            let s =
                                format!("{:?}_[{:?}]", address, metadata.items.get(0).unwrap().contract_name);
                            s
                        } else {
                            let s =
                                format!("UNLABELLED|{:?}|{:?}", address, &transactions.len());
                            s
                        }
                    }
                }
            }
        );

        if label.is_none() && transactions.len() < FAN_OUT_LIMIT {
            walk(
                address,
                &label,
                etherscan_client,
                Arc::clone(&seen),
                current_depth + 1,
                max_depth,
                forward_only,
                backward_only,
            )
            .await?;
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
