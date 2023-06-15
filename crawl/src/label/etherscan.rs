use std::collections::HashMap;

use async_trait::async_trait;
use ethers::types::{Address, Chain};
use ethers_etherscan::{contract::ContractMetadata, Client};

use super::Labeller;

pub struct Etherscan {
    client: Client,
}

impl Default for Etherscan {
    fn default() -> Self {
        Etherscan {
            client: Client::new_from_env(Chain::Mainnet)
                .expect("could not create etherscan client"),
        }
    }
}

#[async_trait]
impl Labeller for Etherscan {
    async fn get_label(&self, address: &Address) -> Option<String> {
        let metadata = self.client.contract_source_code(*address).await;
        match metadata {
            Ok(m) => Some(metadata_to_label(m)),
            Err(e) => {
                println!("Etherscan error: {:?}", e);
                None
            }
        }
    }

    async fn get_labels(&self, addresses: &[Address]) -> HashMap<Address, String> {
        let mut result = HashMap::new();
        for address in addresses {
            if let Some(label) = self.get_label(address).await {
                result.insert(*address, label);
            }
        }
        result
    }
}

fn metadata_to_label(metadata: ContractMetadata) -> String {
    let mut s = String::new();
    for item in metadata.items {
        s.push_str(&item.contract_name);
    }
    println!("Etherscan contract label: {}", s);
    s
}
