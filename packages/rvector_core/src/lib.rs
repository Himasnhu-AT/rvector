use configs::types::{Storage, VectorDocument};
use embeddings::{EmbeddingModel, InitOptions, TextEmbedding};
use storage::storage::StoreData;
// use vector::convert_to_vector;

pub fn check_working() {
    println!("Hello, world!");

    let model: TextEmbedding = TextEmbedding::try_new(InitOptions {
        model_name: EmbeddingModel::BGESmallENV15,
        show_download_progress: true, // set to false to disable download progress
        ..Default::default()
    })
    .unwrap();

    let short_texts = vec![
        "Contribution shall mean any work of authorship, including
      the original version of the Work and any modifications or additions
      to that Work or Derivative Works thereof, that is intentionally
      submitted to Licensor for inclusion in the Work by the copyright owner
      or by an individual or Legal Entity authorized to submit on behalf of
      the copyright owner. For the purposes of this definition, submitted
      means any form of electronic, verbal, or written communication sent
      to the Licensor or its representatives, including but not limited to
      communication on electronic mailing lists, source code control systems,
      and issue tracking systems that are managed by, or on behalf of, the
      Licensor for the purpose of discussing and improving the Work, but
      excluding communication that is conspicuously marked or otherwise
      designated in writing by the copyright owner as Not a Contribution.
",
        "Contributor shall mean Licensor and any individual or Legal Entity
      on behalf of whom a Contribution has been received by Licensor and
      subsequently incorporated within the Work.
",
        "Derivative Works shall mean any work, whether in Source or Object form,
        that is based on (or derived from) the Work and for which the editorial
        revisions, annotations, elaborations, or other modifications represent,
        as a whole, an original work of authorship. For the purposes of this
        License, Derivative Works shall not include works that remain
        separable from, or merely link (or bind by name) to the interfaces of,
        the Work and Derivative Works thereof.",
    ];

    let embeddings = model.embed(short_texts.clone(), None).unwrap();

    // Convert embeddings to f64
    let converted_embeddings: Vec<Vec<f64>> = embeddings
        .into_iter()
        .map(|embedding| embedding.into_iter().map(|x| x as f64).collect())
        .collect();

    // Initialize storage
    let mut storage = Storage::new("./demo.rvdb");

    // Store a vector document
    for (i, embedding) in converted_embeddings.into_iter().enumerate() {
        // println!("Converted embedding {}: {:#?}", i, embedding);
        let document = VectorDocument {
            key: format!("doc{}", i + 1),
            vector: embedding,
            metadata: None,
        };
        storage.store_vector(document);
    }

    // Save storage
    storage.save();

    // Load storage
    storage.load();

    println!("Stored all embeddings successfully.");

    let file: String = "doc1".to_string();

    let retrieved_doc = storage.retrieve_vector(&file).unwrap();

    println!("Retrieved document {}: {:#?}", file, retrieved_doc);

    // Retrieve a vector document
    // if let Some(doc) = storage.retrieve_vector("doc1") {
    //     println!("Retrieved document: {:?}", doc);
    // } else {
    //     println!("Document not found");
    // }

    // Update a vector document
    // let updated_document = VectorDocument {
    //     key: "doc1".to_string(),
    //     vector: vec![4.0, 5.0, 6.0],
    //     metadata: None,
    // };
    // storage.update_vector(updated_document);

    // Delete a vector document
    // storage.delete_document("doc1");

    // List all documents
    // let documents = storage.list_documents();
    // println!("Documents: {:?}", documents);

    // Search documents
    // let search_results = storage.search_documents("query");
    // println!("Search results: {:?}", search_results);
}
