use std::{path::PathBuf, sync::Arc, env::current_exe};

use merkle_hash::{bytes_to_hex, Algorithm, MerkleTree};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exectuable{
    pub path: PathBuf,
    pub checksum: Arc<str>,
}

impl Exectuable {
    pub fn new() -> Self {
        let path = current_exe().expect("Failed to get Current Exectuable Location");
        let tree = MerkleTree::builder(path.to_str().expect("Failed to Resolve Path")).algorithm(Algorithm::Blake3).hash_names(false).build().expect("Failed to Build Merkle Tree");
        let hash = Arc::from(bytes_to_hex(tree.root.item.hash).as_str());     
        Self { path, checksum: hash }
    }
}

impl Default for Exectuable {
    fn default() -> Self {
        Self { path: PathBuf::default(), checksum: Arc::from("") }
    }
}

