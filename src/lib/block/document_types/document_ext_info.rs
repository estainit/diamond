use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct DocExtInfo {
}

impl DocExtInfo {
    pub fn new() -> DocExtInfo {
        DocExtInfo {
        }
    }
}