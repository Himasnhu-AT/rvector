use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VectorDocument {
    pub key: String,
    pub vector: Vec<f64>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct StoreVectorRequest {
    pub key: String,
    pub vector: Vec<f64>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct RetrieveVectorRequest {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVectorRequest {
    pub key: String,
    pub vector: Vec<f64>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDocumentRequest {
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchDocumentsRequest {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct StoreVectorResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct RetrieveVectorResponse {
    pub success: bool,
    pub vector: Option<VectorDocument>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateVectorResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteDocumentResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SearchDocumentsResponse {
    pub success: bool,
    pub results: Vec<VectorDocument>,
    pub message: String,
}
