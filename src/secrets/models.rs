use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub version: i32,
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretBytes {
    pub version: i32,
    pub secret: Vec<i32>,
}

pub type SecretDict = BTreeMap<String, Vec<i32>>;
