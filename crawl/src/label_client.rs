use std::collections::HashMap;

use ethers::types::Address;
use serde_json::Value;

pub async fn get_address_labels(
    addresses: Vec<Address>,
) -> anyhow::Result<HashMap<Address, Option<String>>> {
    // let cache_miss = vec![];
    // Load the first file into a string.
    let text = std::fs::read_to_string("combinedAllLabels.json").unwrap();
    // Parse the string into a dynamically-typed JSON structure.
    let json_cache = serde_json::from_str::<Value>(&text).unwrap();
    for address in addresses {
        let label = json_cache
            .get(&address.to_string())
            .and_then(|v| v.as_object())
            .map(|s| s.get("name"));
        println!("Address: {:?} | Label: {:?}", address, label);
    }
    Ok(HashMap::new())
}
