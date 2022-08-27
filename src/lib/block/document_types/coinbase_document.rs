use crate::lib::custom_types::CMPAIValueT;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::TOutput;
use serde::{Serialize, Deserialize};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CoinbaseDocument {
    pub m_doc_cycle: String,
    pub m_treasury_from: String,
    pub m_treasury_to: String,
    pub m_minted_coins: CMPAIValueT,
    pub m_treasury_incomes: CMPAIValueT,

    pub m_outputs: Vec<TOutput>,
}

impl CoinbaseDocument {
    pub fn new() -> CoinbaseDocument {
        CoinbaseDocument {
            m_doc_cycle: "".to_string(),
            m_treasury_from: "".to_string(),
            m_treasury_to: "".to_string(),
            m_minted_coins: 0,
            m_treasury_incomes: 0,
            m_outputs: vec![],
        }
    }

    // old name was calcDocExtInfoHash
    // old name was calcDocExtInfoHash
    pub fn calc_doc_ext_info_hash(&self, _doc: &Document) -> String
    {
        return "".to_string();
    }

    // old name was hasSignable
    pub fn has_signable(&self, _doc:&Document) ->bool
    {
        return false;
    }

    // old name was veridfyDocSignature
    pub fn verify_doc_signature(&self, _doc:&Document) ->bool
    {
        return true;
    }

    // old name was customValidateDoc
    pub fn custom_validate_doc(&self, _doc:&Document, _block:&Block) ->(bool, String)
    {
        return (true, "".to_string());
    }


}