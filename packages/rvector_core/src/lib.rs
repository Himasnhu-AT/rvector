use configs::types::{Storage, VectorDocument};
use embeddings::embed;
use storage::storage::StoreData;
use vector::convert_to_vector;

pub fn check_working() {
    println!("Hello, world!");
    embed();
    convert_to_vector();
    println!("{}", Storage::working());

    // Initialize storage
    let mut storage = Storage::new("./demo.db");

    // Save storage
    storage.save();

    // Load storage
    storage.load();

    // Store a vector document
    let document = VectorDocument {
        key: "doc1".to_string(),
        vector: vec![1.0, 2.0, 3.0],
        metadata: None,
    };
    storage.store_vector(document);

    // Retrieve a vector document
    if let Some(doc) = storage.retrieve_vector("doc1") {
        println!("Retrieved document: {:?}", doc);
    } else {
        println!("Document not found");
    }

    // Update a vector document
    let updated_document = VectorDocument {
        key: "doc1".to_string(),
        vector: vec![4.0, 5.0, 6.0],
        metadata: None,
    };
    storage.update_vector(updated_document);

    // Delete a vector document
    storage.delete_document("doc1");

    // List all documents
    let documents = storage.list_documents();
    println!("Documents: {:?}", documents);

    // Search documents
    let search_results = storage.search_documents("query");
    println!("Search results: {:?}", search_results);
}
