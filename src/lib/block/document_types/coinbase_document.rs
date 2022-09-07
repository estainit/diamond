use crate::lib::custom_types::{CDocHashT, CMPAIValueT, JSonObject};
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::TOutput;
use serde::{Serialize, Deserialize};
use crate::{ccrypto, constants, cutils, dlog};
use crate::cutils::remove_quotes;
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::{Document, set_document_outputs};

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
    pub fn new() -> Self {
        CoinbaseDocument {
            m_doc_cycle: "".to_string(),
            m_treasury_from: "".to_string(),
            m_treasury_to: "".to_string(),
            m_minted_coins: 0,
            m_treasury_incomes: 0,
            m_outputs: vec![],
        }
    }

    // old name was setByJsonObj
    pub fn set_doc_by_json_obj(&mut self, json_obj: &JSonObject) -> bool {
        if !json_obj["dCycle"].is_null()
        {
            self.m_doc_cycle = remove_quotes(&json_obj["dCycle"]);
        }

        if !json_obj["treasuryFrom"].is_null()
        {
            self.m_treasury_from = remove_quotes(&json_obj["treasuryFrom"]);
        }

        if !json_obj["treasuryTo"].is_null()
        {
            self.m_treasury_to = remove_quotes(&json_obj["treasuryTo"]);
        }

        if !json_obj["mintedCoins"].is_null()
        {
            self.m_minted_coins = json_obj["mintedCoins"].as_u64().unwrap();
        }

        if !json_obj["treasuryIncomes"].is_null()
        {
            self.m_treasury_incomes = json_obj["treasuryIncomes"].as_u64().unwrap();
        }

        if json_obj["outputs"].is_array()
        {
            let outputs = json_obj["outputs"].as_array().unwrap();
            self.set_document_outputs(outputs);
        }

        return true;
    }

    //old_name_was setDocumentOutputs
    pub fn set_document_outputs(&mut self, obj: &Vec<JSonObject>) -> bool
    {
        self.m_outputs = set_document_outputs(obj);
        return true;
    }

    //old_name_was getDocHashableString
    pub fn get_doc_hashable_string(&self, doc: &Document) -> String
    {
        let hash_ables: String = format!(
            "dClass:{},dCycle:{},dLen:{},dType:{},dVer:{},mintedCoins:{},outputs:{},treasuryFrom:{},treasuryIncomes:{},treasuryTo:{}",
            doc.m_doc_class,
            self.m_doc_cycle,
            cutils::padding_length_value(doc.m_doc_length.to_string(), constants::LEN_PROP_LENGTH),
            doc.m_doc_type,
            doc.m_doc_version,
            self.m_minted_coins,
            doc.stringify_outputs(),
            self.m_treasury_from,
            self.m_treasury_incomes,
            self.m_treasury_to,
        );
        return hash_ables;
    }


    pub fn apply_doc_first_impact(
        &self,
        _doc: &Document,
        _block: &Block) -> bool
    {
        // coinbase documents haven't first impact functionalities
        return true;
    }

    //old_name_was getOutputs
    pub fn get_outputs(&self) -> &Vec<TOutput>
    {
        return &self.m_outputs;
    }

    // old name was calcDocHash
    pub fn calc_doc_hash(&self, doc: &Document) -> CDocHashT
    {
        let to_be_hashed_string = self.get_doc_hashable_string(doc);
        dlog(
            &format!("\nHashable string for coinbase block: {}", to_be_hashed_string),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);
        let the_hash = ccrypto::keccak256_dbl(&to_be_hashed_string); // NOTE: absolutely using double hash for more security
        return the_hash;
    }

    // old name was calcDocExtInfoHash
    // old name was calcDocExtInfoHash
    pub fn calc_doc_ext_info_hash(&self, _doc: &Document) -> String
    {
        return "".to_string();
    }

    // old name was hasSignable
    pub fn has_signable(&self, _doc: &Document) -> bool
    {
        return false;
    }

    // old name was veridfyDocSignature
    pub fn verify_doc_signature(&self, _doc: &Document) -> bool
    {
        return true;
    }

    // old name was customValidateDoc
    pub fn custom_validate_doc(&self, _doc: &Document, _block: &Block) -> (bool, String)
    {
        return (true, "".to_string());
    }

    //old_name_was exportDocToJson
    pub fn export_doc_to_json(&self, doc: &Document, ext_info_in_document: bool) -> JSonObject
    {
        let mut j_doc: JSonObject = doc.export_doc_to_json_super(ext_info_in_document);

        if !j_doc["dExtInfo"].is_null()
        {
            j_doc["dExtInfo"] = "".into();
        }

        j_doc["dCycle"] = self.m_doc_cycle.clone().into();
        j_doc["dCDate"] = self.m_doc_cycle.clone().into();
        j_doc["treasuryFrom"] = self.m_treasury_from.clone().into();
        j_doc["treasuryTo"] = self.m_treasury_to.clone().into();
        j_doc["treasuryIncomes"] = self.m_treasury_incomes.into();
        j_doc["mintedCoins"] = self.m_minted_coins.into();
        j_doc["outputs"] = doc.make_outputs_tuples().into();

        return j_doc;
    }
}