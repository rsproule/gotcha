use std::collections::HashMap;

use ethers::types::Address;
use kv::{Config, Store};

use crate::metadock_client;

pub async fn get_address_labels(
    addresses: Vec<Address>,
) -> anyhow::Result<HashMap<Address, Option<String>>> {
    // L2 cache, persistent disk cache
    //

    let mut result = HashMap::new();
    let mut missed = Vec::new();
    for address in addresses {
        let label = get_address_label(address).await;
        if label.is_err() {
            missed.push(address);
        } else {
            println!("Cache hit for {:?}", address);
            if label.as_ref().unwrap().contains("UNLABELLED") {
                result.insert(address, None);
            } else {
                result.insert(address, Some(label.unwrap()));
            }
        }
    }
    let labelled_addresses = metadock_client::get_address_label(missed, 3).await?;
    store_batch(&labelled_addresses)?;
    result.extend(labelled_addresses);
    Ok(result)
}

async fn get_address_label(address: Address) -> anyhow::Result<String> {
    // room for an alternative L2 cache, like reading from a different api
    fetch(address)
}

fn store_batch(labelled_addresses: &HashMap<Address, Option<String>>) -> anyhow::Result<()> {
    let mut cfg = Config::new("./cache/");
    let store = Store::new(cfg)?;
    let bucket = store.bucket::<String, String>(Some("labels"))?;
    for (address, label) in labelled_addresses {
        // if let Some(label) = label {
        let default = format!("UNLABELLED|{:?}", address.to_string());
        let label = label.as_ref().unwrap_or(&default);
        bucket.set(&address.to_string(), label)?;
        // }
    }

    Ok(())
}

fn store(address: Address, label: String) -> anyhow::Result<()> {
    let mut cfg = Config::new("./cache/");

    // Open the key/value store
    let store = Store::new(cfg)?;

    // A Bucket provides typed access to a section of the key/value store
    let bucket = store.bucket::<String, String>(Some("labels"))?;

    let key = address.to_string();
    let value = label;

    // Set test = 123
    bucket.set(&key, &value)?;
    Ok(())
}

fn fetch(address: Address) -> anyhow::Result<String> {
    let mut cfg = Config::new("./cache/");
    let store = Store::new(cfg)?;
    let bucket = store.bucket::<String, String>(Some("labels"))?;
    bucket
        .get(&address.to_string())?
        .ok_or(anyhow::format_err!("Not found"))
}
