use std::collections::HashMap;

use async_trait::async_trait;
use ethers::{types::Address, utils::__serde_json::json};
use serde_json::Number;

use super::Labeller;

struct Metadock {
    client: reqwest::Client,
    url: String,
    chain: String,
    retries: u64,
}

#[async_trait]
impl Labeller for Metadock {
    async fn get_label(&self, address: Address) -> Option<String> {
        let map = self.get_labels(vec![address]).await;
        map.get(&address).map(|s| s.to_string())
    }

    async fn get_labels(&self, addresses: Vec<Address>) -> HashMap<Address, String> {
        let res = Self::do_request(self, addresses).await;
        match res {
            Ok(map) => map,
            Err(e) => match e {
                MetadockError::ResponseError(e) => {
                    if e.code.as_u64().unwrap() == 40000000 {
                        println!("Got rate limit, retrying");
                        // OperationResult::Retry("rate limit")
                    }
                    HashMap::new()
                }
                MetadockError::Unknown(e) => {
                    println!("Internal Metadock error: {:?}", e);
                    HashMap::new()
                }
            },
        }
    }
}

impl Metadock {
    async fn do_request(
        &self,
        addresses: Vec<Address>,
    ) -> Result<HashMap<Address, String>, MetadockError> {
        let json = json!({
            "addresses": addresses,
            "chain": self.chain
        });

        match self
            .client
            .post(&self.url)
            .json(&json)
            .send()
            .await?
            .json::<MetadockResponse>()
            .await?
        {
            MetadockResponse::Error(e) => Err(e.into()),
            MetadockResponse::Success(v) => {
                let mut map = HashMap::new();
                for data in v {
                    let address = data.address.parse::<Address>().unwrap();
                    map.insert(address, data.label);
                }
                Ok(map)
            }
        }
    }
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
enum MetadockResponse {
    Success(Vec<MetadockResponseData>),
    Error(MetadockResponseError),
}

#[derive(serde::Deserialize, Debug)]
struct MetadockResponseData {
    pub address: String,
    pub label: String,
    pub logo: String,
    pub risk: Number,
}
#[derive(serde::Deserialize, Debug)]
pub struct MetadockResponseError {
    pub code: Number,
    pub message: String,
}

impl std::convert::From<reqwest::Error> for MetadockError {
    fn from(err: reqwest::Error) -> Self {
        MetadockError::Unknown(err.to_string())
    }
}
impl std::convert::From<MetadockResponseError> for MetadockError {
    fn from(err: MetadockResponseError) -> Self {
        MetadockError::ResponseError(err)
    }
}
pub enum MetadockError {
    ResponseError(MetadockResponseError),
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_address_label() {
        let addresses: Vec<Address> = vec!["0xb5d85cbf7cb3ee0d56b3bb207d5fc4b82f43f511"
            .parse()
            .unwrap()];

        let metadock_client = Metadock {
            client: reqwest::Client::new(),
            url: "https://extension.blocksec.com/api/v1/address-label".to_string(),
            chain: "eth".to_string(),
            retries: 0,
        };

        // while true {
        let lables = metadock_client.get_labels(addresses.clone()).await;
        println!("Labels: {:?}", lables);
        // }
    }
}
