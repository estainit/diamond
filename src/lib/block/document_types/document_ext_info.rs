use serde::{Serialize, Deserialize};
use crate::lib::custom_types::VVString;
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
}