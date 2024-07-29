use configs::types::{Storage, VectorDocument};
use embeddings::embed;
use storage::storage::StoreData;
use vector::convert_to_vector;

pub fn check_working() {
    println!("Hello, world!");
    embed();
    convert_to_vector();
    println!("?{}", Storage::working());
}
