use configs::types::{Storage, VectorDocument};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

pub trait StoreData {
    fn working() -> &'static str;
    fn new(path: &str) -> Self;
    fn save(&self);
    fn load(&mut self);
    fn store_vector(&mut self, document: VectorDocument);
    fn retrieve_vector(&self, key: &str) -> Option<&VectorDocument>;
    fn update_vector(&mut self, document: VectorDocument);
    fn delete_document(&mut self, key: &str);
    fn list_documents(&self) -> Vec<&VectorDocument>;
    fn search_documents(&self, query: &str) -> Vec<&VectorDocument>;
}

impl StoreData for Storage {
    fn working() -> &'static str {
        "Hello, world! from package storage"
    }

    fn new(path: &str) -> Self {
        println!("Creating new storage at path: {}", path);
        Storage {
            path: path.to_string(),
            data: HashMap::new(),
        }
    }

    fn save(&self) {
        println!("Saving storage to file");
        let serialized = serde_json::to_string(&self.data).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)
            .unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
    }

    fn load(&mut self) {
        println!("Loading storage from file");
        let mut file = OpenOptions::new().read(true).open(&self.path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        self.data = serde_json::from_str(&content).unwrap();
    }

    fn store_vector(&mut self, document: VectorDocument) {
        println!("Storing vector document with key: {}", document.key);
        self.data.insert(document.key.clone(), document);
    }

    fn retrieve_vector(&self, key: &str) -> Option<&VectorDocument> {
        println!("Retrieving vector document with key: {}", key);
        self.data.get(key)
    }

    fn update_vector(&mut self, document: VectorDocument) {
        println!("Updating vector document with key: {}", document.key);
        self.data.insert(document.key.clone(), document);
    }

    fn delete_document(&mut self, key: &str) {
        println!("Deleting vector document with key: {}", key);
        self.data.remove(key);
    }

    fn list_documents(&self) -> Vec<&VectorDocument> {
        println!("Listing all vector documents");
        self.data.values().collect()
    }

    fn search_documents(&self, query: &str) -> Vec<&VectorDocument> {
        println!("Searching for documents with query: {}", query);
        self.data
            .values()
            .filter(|doc| doc.key.contains(query))
            .collect()
    }
}
