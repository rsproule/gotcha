use async_trait::async_trait;
use ethers::types::Address;
use std::collections::HashMap;
pub mod cache;
mod etherscan;
mod metadock;

#[async_trait]
trait Labeller {
    async fn get_label(&self, address: &Address) -> Option<String>;

    async fn get_labels(&self, addresses: &[Address]) -> HashMap<Address, String>;
}

#[async_trait]
trait Cache<K, V> {
    async fn get(&self, key: K) -> Option<V>;

    async fn set(&self, key: K, value: V) -> Option<V>;
}
