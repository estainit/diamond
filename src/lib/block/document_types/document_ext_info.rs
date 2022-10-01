use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::cutils::remove_quotes;
use crate::lib::custom_types::{JSonObject, VString, VVString};
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

    pub fn load_from_json(j_obj: &JSonObject) -> (bool, Self)
    {
        let mut signatures: VVString = vec![];
        for a_sig_vec in j_obj["signatures"].as_array().unwrap()
        {
            signatures.push(
                a_sig_vec
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| remove_quotes(x))
                    .collect::<VString>());
        }
        let (status, unlock_set) = UnlockSet::load_from_json(&j_obj["uSet"]);
        if !status
        {
            return (false, Self::new());
        }
        let out: Self = Self {
            m_unlock_set: unlock_set,
            m_signatures: signatures,
        };
        return (true, out);
    }
}