use crate::lib::custom_types::{CDocHashT, JSonObject};
use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::{application, ccrypto, constants, cutils, dlog};
use crate::cutils::remove_quotes;
use crate::lib::block::block_types::block::Block;
use crate::lib::messaging_protocol::dispatcher::make_a_packet;
use crate::lib::parsing_q_handler::queue_pars::EntryParsingResult;
use crate::lib::sending_q_handler::sending_q_handler::push_into_sending_q;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CoinbaseBlock
{
    pub m_cycle: String,
}

impl CoinbaseBlock {
    pub fn new() -> Self {
        CoinbaseBlock {
            m_cycle: "".to_string()
        }
    }

    pub fn set_block_by_json_obj(&mut self, obj: &JSonObject) -> bool
    {
        self.m_cycle = remove_quotes(&obj["bDocs"].as_array().unwrap()[0]["dCycle"]);
        return true;
    }

    // old name was handleReceivedBlock
    pub fn handle_received_block(&self, block_super: &Block) -> EntryParsingResult
    {
        let block_identifier: String = block_super.get_block_identifier();
        let error_message: String;
        let en_pa_res = self.validate_coinbase_block(block_super);
        dlog(
            &format!(
                "Received validate CoinbaseBlock result: status({}) should_purge_record({})",
                en_pa_res.m_status,
                en_pa_res.m_should_purge_record,
            ),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);

        if !en_pa_res.m_status
        {
            // do something (e.g. bad reputation log for sender neighbor)
            return en_pa_res;
        }

        dlog(
            &format!(
                "dummy log pre add to DAG a CoinbaseBlock: {}", cutils::controlled_block_stringify(&block_super)),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);

        block_super.add_block_to_dag();

        block_super.post_add_block_to_dag();


        // broadcast block to neighbors
        if application().is_in_current_cycle(&block_super.m_block_creation_date)
        {
            let mut block_body = block_super.safe_stringify_block(true);
            block_body = ccrypto::b64_encode(&block_body);

            let (_code, body) = make_a_packet(
                vec![
                    json!({
                "cdType": block_super.m_block_type,
                "cdVer": constants::DEFAULT_CARD_VERSION,
                "bHash": block_super.get_block_hash(),
                "block": block_body,
            }),
                ],
                constants::DEFAULT_PACKET_TYPE,
                constants::DEFAULT_PACKET_VERSION,
                application().now(),
            );
            dlog(
                &format!("prepared coinbase packet, before insert into DB code({}) {}", block_super.m_block_hash, body),
                constants::Modules::App,
                constants::SecLevel::Info);

            let status = push_into_sending_q(
                block_super.m_block_type.as_str(),
                block_super.m_block_hash.as_str(),
                &body,
                &format!("Broadcasting the confirmed coinbase {} ", cutils::hash16c(&block_super.get_block_hash())),
                &vec![],
                &vec![],
                false,
            );

            dlog(
                &format!("coinbase push res({})", status),
                constants::Modules::App,
                constants::SecLevel::Info);
        }
        error_message = format!(
            "coinbase block confirmed and inserted to local DAG {}",
            block_identifier, );
        dlog(
            &error_message,
            constants::Modules::CB,
            constants::SecLevel::Info);
        return EntryParsingResult {
            m_status: true,
            m_should_purge_record: true,
            m_message: error_message,
        };
    }

    pub fn get_block_hashable_string(&self, block: &Block) -> String
    {
        // in order to have almost same hash! we sort the attribiutes alphabeticaly
        let block_hashables: String = format!(
            "bAncestors:{},bCDate:{},bDocsRootHash:{},bLen:{},bType:{},bVer:{},cycle:{},net:{}",
            serde_json::to_string(&block.m_block_ancestors).unwrap(),
            block.m_block_creation_date,
            block.m_block_documents_root_hash, // note that we do not put the docsHash directly in block hash, instead using docsHash-merkle-root-hash
            block.m_block_length,
            block.m_block_type,
            block.m_block_version,
            self.m_cycle,
            block.m_block_net
        );
        return block_hashables;
    }

    pub fn calc_block_hash(&self, block: &Block) -> String
    {
        let block_hash_ables: String = self.get_block_hashable_string(block);

        // clonedTransactionsRootHash: block.clonedTransactionsRootHash,
        // note that we do not put the clonedTransactions directly in block hash,
        // instead using clonedTransactions-merkle-root-hash

        dlog(
            &format!("The Coinbase! block hashable: {}", block_hash_ables),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return ccrypto::keccak256(&block_hash_ables);
    }

    pub fn calc_block_ext_root_hash(&self, _block: &Block) -> (bool, CDocHashT)
    {
        return (true, "".to_string());
    }

    pub fn export_block_to_json(
        &self,
        _block: &Block,
        parent_json_obj: &mut JSonObject,
        _ext_info_in_document: bool) -> JSonObject
    {

        // maybe remove add some item in object
        if !parent_json_obj["bExtInfo"].is_null()
        {
            parent_json_obj["bExtInfo"] = "".into();
        }

        if !parent_json_obj["bExtHash"].is_null()
        {
            parent_json_obj["bExtHash"] = "".into();
        }

        if !parent_json_obj["fVotes"].is_null()
        {
            parent_json_obj["fVotes"] = "".into();
        }

        if !parent_json_obj["signals"].is_null()
        {
            parent_json_obj["signals"] = "".into();
        }

        if !parent_json_obj["backer"].is_null()
        {
            parent_json_obj["backer"] = "".into();
        }

        parent_json_obj["bLen"] = constants::LEN_PROP_PLACEHOLDER.into();
        return parent_json_obj.clone();
    }

    /*
     String CoinbaseBlock::safe_stringify_block(const bool ext_info_in_document) const
     {
       JSonObject block = export_block_to_json(ext_info_in_document);

       // recaluculate block final length
       String tmp_stringified = cutils::serializeJson(block);
       block["bLen"] = cutils::padding_length_value(tmp_stringified.length());

       String out = cutils::serializeJson(block);
       CLog::log("Safe sringified block(Coinbase) Block(" + cutils::hash8c(m_block_hash) + ") length(" + String::number(out.length()) + ") the block: " + out, "app", "trace");

       return out;
     }

     BlockLenT CoinbaseBlock::calcBlockLength(const JSonObject& block_obj) const
     {
       return cutils::serializeJson(block_obj).length();
     }
    */

    // old name was controlBlockLength
    pub fn control_block_length(&self, block: &Block) -> bool
    {
        let stringified_block = block.safe_stringify_block(true);
        if stringified_block.len() != block.m_block_length
        {
            dlog(
                &format!(
                    "Mismatch coinbase block length Block({})  local length({}) remote length({}) stringyfied remote block: {}",
                    cutils::hash8c(&block.m_block_hash),
                    stringified_block.len(),
                    block.m_block_length,
                    stringified_block),
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return false;
        }

        return true;
    }

    /*
        String CoinbaseBlock::stringify_block_ext_info() const
        {
          return "";
        }

        */
}