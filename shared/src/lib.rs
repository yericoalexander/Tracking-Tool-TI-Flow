use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashRequest {
    pub id: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashResponse {
    pub id: String,
    pub hash: String,
}

impl HashRequest {
    pub fn new(data: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            data,
        }
    }
}