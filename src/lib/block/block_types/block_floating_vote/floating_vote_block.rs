use serde_json::json;
use serde::{Serialize, Deserialize};
use substring::Substring;
use crate::{application, ccrypto, constants, cutils, dlog, machine};
use crate::cutils::remove_quotes;
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document_ext_info::DocExtInfo;
use crate::lib::block::node_signals_handler::get_machine_signals;
use crate::lib::custom_types::{CBlockHashT, JSonObject, SharesPercentT};
use crate::lib::services::dna::dna_handler::get_machine_shares;
use crate::lib::services::society_rules::society_rules::get_min_share_to_allowed_issue_f_vote;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::safe_stringify_unlock_set;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FloatingVoteBlock
{
    pub m_vote_category: String,
    pub m_vote_confidence: f64,
    pub m_block_vote_data: JSonObject,
}

impl FloatingVoteBlock {
    pub fn new() -> Self
    {
        Self {
            m_vote_category: "".to_string(),
            m_vote_confidence: 0.0,
            m_block_vote_data: json!({}),
        }
    }

    pub fn set_block_by_json_obj(&mut self, obj: &JSonObject) -> bool
    {
        if !obj["bVCat"].is_null() {
            self.m_vote_category = remove_quotes(&obj["bVCat"]);
        }

        if !obj["bConfidence"].is_null() {
            self.m_vote_confidence = remove_quotes(&obj["bConfidence"]).parse::<f64>().unwrap();
        }


        true
    }

    // old name was calcBlockExtRootHash
    pub fn calc_block_ext_root_hash(&self, block: &Block) -> (bool, String)
    {
        let ext_info_hash_ables: String = format!(
            "uSet:{},signatures:{}",
            safe_stringify_unlock_set(&block.m_block_ext_info[0][0].m_unlock_set),
            serde_json::to_string(&block.m_block_ext_info[0][0].m_signatures).unwrap()
        );
        let the_hash = ccrypto::keccak256(&ext_info_hash_ables);
        dlog(
            &format!(
                "Floating Vote Block Hash-ables for ext hash: {}, Regenerated Ext hash: {}",
                ext_info_hash_ables,
                the_hash),
            constants::Modules::App,
            constants::SecLevel::Info);
        (true, the_hash)
    }

    // old name was getBlockHashableString
    pub fn get_block_hashable_string(&self, block: &Block) -> String
    {
        let block_hash_ables: String = format!(
            "bAncestors:{},bBacker:{},bCDate:{},bExtHash:{},bLen:{},bNet:{},bSignals:{},bType:{},bVer:{},bVCat:{},bVConfidence:{},bVData:{}",
            serde_json::to_string(&block.m_block_ancestors).unwrap(),
            block.m_block_backer,
            block.m_block_creation_date,
            block.m_block_ext_root_hash,    // note that we do not put the segwits directly in block hash, instead using segwits-merkle-root-hash
            block.m_block_length,
            block.m_block_net,
            serde_json::to_string(&block.m_block_signals).unwrap(),
            block.m_block_type,
            block.m_block_version,
            self.m_vote_category,
            self.m_vote_confidence,
            serde_json::to_string(&self.m_block_vote_data).unwrap(),
        );
        return block_hash_ables;
    }

    pub fn calc_block_hash(&self, block: &Block) -> CBlockHashT
    {
        let hashable_block = self.get_block_hashable_string(block);
        let the_hash = ccrypto::keccak256(&hashable_block);
        dlog(
            &format!("FV-block-hash-ables: {}, the hash: {}", hashable_block, the_hash),
            constants::Modules::App,
            constants::SecLevel::Info);
        return the_hash;
    }

    /*
    bool FloatingVoteBlock::controlBlockLength() const
    {
      String stringyfied_block = safeStringifyBlock(false);
      if (static_cast<BlockLenT>(stringyfied_block.length()) != m_block_length)
      {
        CLog::log("Mismatch floating vote block length Block(" + cutils::hash8c(m_block_hash) + ") local length(" + String::number(stringyfied_block.length()) + ") remote length(" + String::number(m_block_length) + ") stringyfied_block:" + stringyfied_block, "sec", "error");
        return false;
      }
      return true;
    }

    BlockLenT FloatingVoteBlock::calcBlockLength(const JSonObject& block_obj) const
    {
      return Block::calcBlockLength(block_obj);
    }

    JSonObject FloatingVoteBlock::exportBlockToJSon(const bool ext_info_in_document) const
    {
      JSonObject Jblock = Block::exportBlockToJSon(ext_info_in_document);

      Jblock["bLen"] = cutils::paddingLengthValue(calcBlockLength(Jblock));

      return Jblock;
    }

    String FloatingVoteBlock::safeStringifyBlock(const bool ext_info_in_document) const
    {
      JSonObject block = exportBlockToJSon(ext_info_in_document);

      // maybe remove add some item in object


      // recaluculate block final length
      String tmp_stringified = cutils::serializeJson(block);
      block["bLen"] = cutils::paddingLengthValue(tmp_stringified.length());

      String out = cutils::serializeJson(block);
      CLog::log("Safe sringified block(floating vote) Block(" + cutils::hash8c(m_block_hash) + ") length(" + String::number(out.length()) + ") the block: " + out, "app", "trace");

      return out;
    }

    /**
    * @brief NormalBlock::handleReceivedBlock
    * @return <status, should_purge_record>
    */
    std::tuple<bool, bool> FloatingVoteBlock::handleReceivedBlock() const
    {
      return {false, true};
    }

    String FloatingVoteBlock::dumpBlock() const
    {
      // firsdt call parent dump
      String out = Block::dumpBlock();

      // then child dumpping
      out += "\n in child";
      return out;
    }


    */

    // old name was getSignMsgBFVote
    pub fn get_sign_msg_b_f_vote(&self,block: &Block) -> String
    {
        let fv_block_sign_ables: String = format!(
            "bAncestors:{:?},bCDate:{},bConfidence:{},bNet:{},bType:{},bVer:{},bVoteData:{}",
            block.m_block_ancestors,
            block.m_block_creation_date,
            block.m_if_floating_vote_block.m_vote_confidence,
            block.m_block_net,
            block.m_block_type,
            block.m_block_version,
            block.m_if_floating_vote_block.m_block_vote_data,
        );
        let mut tobe_signed = ccrypto::keccak256(&fv_block_sign_ables);
        tobe_signed = tobe_signed.substring(0, constants::SIGN_MSG_LENGTH as usize).to_string();

        dlog(
            &format!("FV-block-sign-ables: {}, tobe_signed: {}", fv_block_sign_ables, tobe_signed),
            constants::Modules::App,
            constants::SecLevel::Info);

        return tobe_signed;
    }

    // old name was createFVoteBlock
    pub fn create_floating_vote_block(
        uplink: &String,  // the block which we are voting for
        block_category: &String,
        vote_data: &JSonObject,
        cDate: &String) -> (bool, Block)
    {
        dlog(
            &format!(
                "Create Floating Vote Block uplink({}) block Cat({}) cDate({}) vote Data: {:?}",
                cutils::hash8c(uplink),
                block_category,
                cDate,
                vote_data
            ),
            constants::Modules::App,
            constants::SecLevel::Info);

        let now_ = application().now();
        let (_backer_address, _shares, percentage) = get_machine_shares(&now_);
        let min_share: SharesPercentT = get_min_share_to_allowed_issue_f_vote(cDate);
        if percentage < min_share
        {
            dlog(
                &format!(
                    "Machine hasn't sufficient shares ({} < {}) to issue a floating vote for any collision on block Uplink({}) \
              Category({}) cDate({}) vote Data: {}",
                    percentage,
                    min_share,
                    cutils::hash8c(uplink),
                    block_category,
                    cDate,
                    vote_data),
                constants::Modules::App,
                constants::SecLevel::TmpDebug);


            // TODO: push fVote info to machine buffer in order to broadcast to network in first generated block.
            // therefore even machines with small amount of shares will vote and pay the vote cost and won't whelm the network
            return (false, Block::new());
        }

        let j_block: JSonObject = json!({
            "bHash": constants::HASH_ZEROS_PLACEHOLDER,
            "bType": constants::block_types::FLOATING_VOTE,
            "bVCat": block_category,
            "bConfidence": percentage,
            "bAncestors": vec![uplink],
            "bCDate": application().now(),
            "bSignals": get_machine_signals(),
            "bVoteData": vote_data});

        let (status, mut block) = Block::load_block(&j_block);

        // create floating vote block
        let tobe_signed = block.m_if_floating_vote_block.get_sign_msg_b_f_vote(&block);
        let (status, backer, uSet, signatures) =
            machine().sign_by_machine_key(&tobe_signed, 0);
        let mut ext_info = DocExtInfo::new();
        ext_info.m_unlock_set = uSet;
        ext_info.m_signatures = vec![signatures];

        block.m_block_ext_info = vec![vec![ext_info]];

        block.m_block_backer = backer;

        let (status_ext, fv_b_ext_hash) = block.calc_block_ext_root_hash();
        block.m_block_ext_root_hash = fv_b_ext_hash;
        block.set_block_hash(&block.calc_block_hash());

        return (true, block);
    }
}