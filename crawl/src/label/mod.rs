use async_trait::async_trait;
use ethers::types::Address;
use std::collections::HashMap;
mod metadock;

#[async_trait]
trait Labeller {
    async fn get_label(&self, address: Address) -> Option<String>;

    async fn get_labels(&self, addresses: Vec<Address>) -> HashMap<Address, String>;
}
