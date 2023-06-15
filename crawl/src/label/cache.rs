use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use ethers::types::Address;
use kv::{Bucket, Config, Store};
use tokio::sync::Mutex;

use super::{etherscan::Etherscan, metadock::Metadock, Cache, Labeller};

pub struct LabelCache<'a> {
    bucket: Arc<Mutex<Bucket<'a, String, String>>>,
    fetchers: Vec<Box<dyn Labeller + Send + Sync>>,
}

#[async_trait]
impl Cache<String, String> for LabelCache<'_> {
    async fn get(&self, key: String) -> Option<String> {
        let bucket = self.bucket.lock().await;
        match bucket.get(&key) {
            Ok(value) => value,
            Err(_) => None,
        }
    }

    async fn set(&self, key: String, value: String) -> Option<String> {
        let bucket = self.bucket.lock().await;
        match bucket.set(&key, &value) {
            Ok(value) => value,
            Err(_) => None,
        }
    }
}

impl LabelCache<'_> {
    pub fn new() -> Self {
        let cfg = Config::new("./cache/");
        let store = Store::new(cfg).unwrap();
        let bucket = Arc::new(Mutex::new(
            store.bucket::<String, String>(Some("labels")).unwrap(),
        ));
        LabelCache {
            bucket: Arc::clone(&bucket),
            fetchers: vec![Box::<Metadock>::default(), Box::<Etherscan>::default()],
        }
    }

    pub async fn get_labels(&self, addresses: Vec<Address>) -> HashMap<Address, String> {
        let mut result = HashMap::new();
        let mut missed = Vec::new();
        for address in addresses {
            let cache_key = format!("{:#?}", address);
            let label = self.get(cache_key).await;
            match label {
                Some(label) => {
                    // TODO: This is a hack to avoid the "UNLABELLED" label
                    if label != "UNLABELLED" {
                        println!("Cache hit: {}", label);
                        result.insert(address, label);
                    }
                }
                None => missed.push(address),
            }
        }

        if !missed.is_empty() {
            // HACK: chop to the first 100 or just the length
            let chopped_missed = if missed.len() > 50 {
                missed.clone()[0..50].to_vec()
            } else {
                missed.clone()
            };
            println!("Fetching {} labels from the network", chopped_missed.len());
            for fetcher in self.fetchers.iter() {
                let labelled_addresses = fetcher.get_labels(&chopped_missed).await;
                for (k, v) in &labelled_addresses {
                    let cache_key = format!("{:#?}", k);
                    // NOTE: This does not do any negative caching, which we probably want to do.
                    self.set(cache_key, v.to_string()).await;
                }
                result.extend(labelled_addresses);
            }
            // negative cache. problem is this cache key does not check per fetcher
            // for address in missed {
            //     let cache_key = format!("{:#?}", address);
            //     // i dont really like this string, but it's better than nothing
            //     self.set(cache_key, "UNLABELLED".to_string()).await;
            // }
        }
        result
    }
}
