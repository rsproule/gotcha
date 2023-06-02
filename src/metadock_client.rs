use std::{thread, time::Duration};

use ethers::{types::Address, utils::__serde_json::json};

#[async_recursion::async_recursion]
pub async fn get_address_label(addresses: Vec<Address>) -> anyhow::Result<Vec<Option<String>>> {
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
    println!("Response: {:?}", resp);
    let status = resp.get("code").and_then(|v| v.as_u64());
    let opt = resp.as_array().map(|arr| {
        arr.iter()
            .map(|obj| {
                obj.get("label")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect()
    });
    // hacky retry logic, need to model these errors properly
    if let Some(s) = status {
        if s == 40000000 {
            println!("Got rate limit, retrying");
            thread::sleep(Duration::from_secs(1));
            return get_address_label(addresses).await;
        }
    }

    match opt {
        Some(labels) => Ok(labels),
        None => {
            println!("Failed to get label for addresses: {}", resp);
            Ok(vec![])
        }
    }
}
