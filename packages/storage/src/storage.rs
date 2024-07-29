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

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_working() {
        assert_eq!(Storage::working(), "Hello, world! from package storage");
    }

    #[test]
    fn test_new() {
        let storage = Storage::new("./demo.db");
        assert_eq!(storage.path, "./demo.db");
        assert_eq!(storage.data.len(), 0);
    }

    #[test]
    fn test_save_and_load() {
        let mut storage = Storage::new("./demo.db");
        storage.store_vector(VectorDocument {
            key: "document1".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: None,
        });

        storage.save();

        let mut loaded_storage = Storage::new("./demo.db");
        loaded_storage.load();

        assert_eq!(loaded_storage.data.len(), 1);
        assert_eq!(loaded_storage.retrieve_vector("document1").is_some(), true);
    }

    #[test]
    fn test_store_and_retrieve_vector() {
        let mut storage = Storage::new("./demo.db");
        let document = VectorDocument {
            key: "document1".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: None,
        };

        storage.store_vector(document.clone());

        let retrieved_document = storage.retrieve_vector("document1");

        assert_eq!(retrieved_document, Some(&document));
    }

    #[test]
    fn test_update_vector() {
        let mut storage = Storage::new("./demo.db");
        let document = VectorDocument {
            key: "document1".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: None,
        };

        storage.store_vector(document.clone());

        let updated_document = VectorDocument {
            key: "document1".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: None,
        };

        storage.update_vector(updated_document.clone());

        let retrieved_document = storage.retrieve_vector("document1");

        assert_eq!(retrieved_document, Some(&updated_document));
    }

    #[test]
    fn test_delete_document() {
        let mut storage = Storage::new("./demo.db");
        let document = VectorDocument {
            key: "document1".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: None,
        };

        storage.store_vector(document.clone());

        storage.delete_document("document1");

        let retrieved_document = storage.retrieve_vector("document1");

        assert_eq!(retrieved_document, None);
    }

    #[test]
    fn test_list_documents() {
        let mut storage = Storage::new("./demo.db");
        let document1 = VectorDocument {
            key: "document1".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: None,
        };
        let document2 = VectorDocument {
            key: "document2".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: None,
        };

        storage.store_vector(document1.clone());
        storage.store_vector(document2.clone());

        let documents = storage.list_documents();

        assert_eq!(documents.len(), 2);
        assert_eq!(documents.contains(&&document1), true);
        assert_eq!(documents.contains(&&document2), true);
    }

    #[test]
    fn test_search_documents() {
        let mut storage = Storage::new("./demo.db");
        let document1 = VectorDocument {
            key: "document1".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: None,
        };
        let document2 = VectorDocument {
            key: "document2".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: None,
        };
        let document3 = VectorDocument {
            key: "another_document".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: None,
        };

        storage.store_vector(document1.clone());
        storage.store_vector(document2.clone());

        let search_results = storage.search_documents("document");

        assert_eq!(search_results.len(), 2);
        assert_eq!(search_results.contains(&&document1), true);
        assert_eq!(search_results.contains(&&document2), true);
        assert_eq!(search_results.contains(&&document3), false);
    }
}
