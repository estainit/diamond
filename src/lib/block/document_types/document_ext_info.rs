use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::lib::custom_types::{JSonObject, VVString};
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_set::UnlockSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocExtInfo {
    pub m_unlock_set: UnlockSet,
    pub m_signatures: VVString,
}

impl DocExtInfo {
    #[allow(unused, dead_code)]
    pub fn new() -> Self {
        DocExtInfo {
            m_unlock_set: UnlockSet::new(),
            m_signatures: vec![],
        }
    }

    pub fn export_to_json(&self) -> JSonObject
    {
        let out = json!({
           "uSet": self.m_unlock_set.export_to_json(),
           "signatures": self.m_signatures,
        });
        return out;
    }


}