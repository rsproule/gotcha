use std::{collections::HashMap, thread, time::Duration};

use ethers::{types::Address, utils::__serde_json::json};

#[async_recursion::async_recursion]
pub async fn get_address_label(
    addresses: Vec<Address>,
    tries: u64,
) -> anyhow::Result<HashMap<Address, Option<String>>> {
    let client = reqwest::Client::new();
    let json = json!({
        "addresses": addresses,
        "chain": "eth".to_string()
    });
    let resp = client
        .post("https://extension.blocksec.com/api/v1/address-label")
        .json(&json)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    // println!("Response: {:?}", resp);
    let status = resp.get("code").and_then(|v| v.as_u64());
    let opt: Option<HashMap<Address, Option<String>>> = resp.as_array().map(|arr| {
        arr.iter()
            .map(|obj| {
                let address: Option<Address> = obj
                    .get("address")
                    .and_then(|v| v.as_str())
                    .map(|s| s.parse().ok())
                    .and_then(|v| v);
                let label = obj
                    .get("label")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                (address.unwrap(), label)
            })
            .collect()
    });

    // hacky retry logic, need to model these errors properly
    if let Some(s) = status {
        if s == 40000000 {
            println!("Got rate limit, retrying");
            thread::sleep(Duration::from_secs(2 ^ tries));
            return get_address_label(addresses, tries + 1).await;
        }
    }

    match opt {
        Some(mut labels) => {
            for address in addresses {
                // search if this address is in the vec
                if labels.get(&address).is_none() {
                    labels.insert(address, None);
                }
            }
            Ok(labels)
        }
        None => {
            println!("Failed to get label for addresses: {}", resp);
            Ok(HashMap::new())
        }
    }
}
