use ethers::{types::Address, utils::__serde_json::json};

pub async fn get_address_label(address: Address) -> anyhow::Result<Option<String>> {
    let client = reqwest::Client::new();
    let json = json!({
        "addresses": vec![address],
        "chain": "eth".to_string()
    });
    let resp = client
        .post("https://extension.blocksec.com/api/v1/address-label")
        .json(&json)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    match resp[0]["label"].as_str() {
        Some(label) => Ok(Some(label.to_string())),
        None => Ok(None),
    }
}
