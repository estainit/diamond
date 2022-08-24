use crate::lib::custom_types::CMPAIValueT;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::TOutput;
use serde::{Serialize, Deserialize};

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
}