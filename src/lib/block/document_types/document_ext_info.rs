use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct DocExtInfo {
}

impl DocExtInfo {
    #[allow(unused, dead_code)]
    pub fn new() -> DocExtInfo {
        DocExtInfo {
        }
    }
}