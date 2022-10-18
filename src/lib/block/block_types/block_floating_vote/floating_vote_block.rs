use std::collections::HashMap;
use serde_json::json;
use serde::{Serialize, Deserialize};
use substring::Substring;
use crate::{application, ccrypto, constants, cutils, dlog, machine};
use crate::ccrypto::ecdsa_verify_signature;
use crate::cutils::remove_quotes;
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document_ext_info::DocExtInfo;
use crate::lib::block::node_signals_handler::get_machine_signals;
use crate::lib::custom_types::{CBlockHashT, JSonObject, SharesPercentT};
use crate::lib::parsing_q_handler::queue_pars::EntryParsingResult;
use crate::lib::services::dna::dna_handler::{get_an_address_shares, get_machine_shares};
use crate::lib::services::society_rules::society_rules::get_min_share_to_allowed_issue_f_vote;
use crate::lib::transactions::basic_transactions::coins::suspect_trx_handler::add_suspect_transaction;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{safe_stringify_unlock_set, validate_sig_struct};

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

    // old name was validateFVoteBlock
    pub fn validate_f_vote_block(&self, block_super: &Block) -> EntryParsingResult
    {
        let validate_msg: String;

        // control shares/confidence
        let (_shares_count, issuer_shares_percentage) = get_an_address_shares(
            &block_super.get_block_backer(),
            &block_super.get_creation_date());
        if block_super.get_block_confidence() != issuer_shares_percentage
        {
            validate_msg = format!(
                "fVote({}) was rejected! because of wrong confidence({})!=local({})",
                cutils::hash6c(&block_super.get_block_hash()),
                block_super.get_block_confidence(),
                issuer_shares_percentage);
            dlog(
                &validate_msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return EntryParsingResult {
                m_status: false,
                m_should_purge_record: true,
                m_message: validate_msg,
            };
        }

        // control signature
        let is_valid_unlock = validate_sig_struct(
            &block_super.m_block_ext_info[0][0].m_unlock_set,
            &block_super.get_block_backer(),
            &HashMap::new());
        if is_valid_unlock != true
        {
            validate_msg = format!("Invalid given unlock structure for bVote({})", cutils::hash6c(&block_super.get_block_hash()));
            dlog(
                &validate_msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            dlog(
                &validate_msg,
                constants::Modules::Trx,
                constants::SecLevel::Error);
            return EntryParsingResult {
                m_status: false,
                m_should_purge_record: true,
                m_message: validate_msg,
            };
        }

        let sign_msg = self.get_sign_msg_b_f_vote(block_super);
        for signature_inx in 0..block_super.m_block_ext_info[0][0].m_unlock_set.m_signature_sets.len()
        {
            let verify_res = ecdsa_verify_signature(
                &block_super.m_block_ext_info[0][0].m_unlock_set.m_signature_sets[signature_inx].m_signature_key,
                &sign_msg,
                &block_super.m_block_ext_info[0][0].m_signatures[signature_inx][0],
            );
            if verify_res != true
            {
                validate_msg = format!("Invalid given signature for bVote({})", cutils::hash6c(&block_super.get_block_hash()));
                dlog(
                    &validate_msg,
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return EntryParsingResult {
                    m_status: false,
                    m_should_purge_record: true,
                    m_message: validate_msg,
                };
            }
        }

        validate_msg = format!("Received fVote({}) is valid", cutils::hash6c(&block_super.get_block_hash()));
        dlog(
            &validate_msg,
            constants::Modules::Trx,
            constants::SecLevel::Info);
        return EntryParsingResult {
            m_status: true,
            m_should_purge_record: true,
            m_message: validate_msg,
        };
    }


    // old name was handleReceivedBlock
    pub fn handle_received_block(&self, block_super: &Block) -> EntryParsingResult
    {


// static handleReceivedFVoteBlock(args) {
//         let msg;
//         let block = args.payload;
//         let receive_date = _.has(args, 'receive_date') ? args.receive_date : utils.getNow()

        dlog(
            &format!(
                "******** handle Received floating Vote block{} {}",
                block_super.get_block_identifier(),
                block_super.safe_stringify_block(true)),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        let validate_res = self.validate_f_vote_block(block_super);
        if !validate_res.m_status
        {
            // do something
            return validate_res;
        }

        // record in dag
        dlog(
            &format!(
                "Add a valid FloatingVote{} to DAG )",
                block_super.get_block_identifier()),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);


        block_super.add_block_to_dag();
        block_super.post_add_block_to_dag();


        // special treatment based on fVote Category
        if self.m_vote_category == constants::float_blocks_categories::TRANSACTION
        {
            // also insert suspicious transactions
            let double_spends = &self.m_block_vote_data["doubleSpends"];
            dlog(
                &format!("Inserting double-Spends in db: {:?}", double_spends),
                constants::Modules::Trx,
                constants::SecLevel::Info);

            for a_coin in double_spends["coinsCodes"].as_array().unwrap()
            {
                let coin_code = remove_quotes(a_coin);
                for a_dbl_spend_trx in double_spends[coin_code.clone()].as_array().unwrap()
                {
                    // console.log(aTx);
                    add_suspect_transaction(
                        &block_super.get_block_backer(),
                        &block_super.get_creation_date(),
                        &coin_code,
                        &block_super.get_block_hash(),
                        &remove_quotes(&a_dbl_spend_trx["spendBlockHash"]),
                        &remove_quotes(&a_dbl_spend_trx["spendDocHash"]),
                        &remove_quotes(&a_dbl_spend_trx["spendDate"]),
                        remove_quotes(&a_dbl_spend_trx["spendOrder"]).parse::<i32>().unwrap(),
                    );
                }
            }
        } else if self.m_vote_category == constants::float_blocks_categories::FLEXIBLE_NAME_SERVICE
        {
            // const flensHandler = require('../../contracts/flens-contract/flens-handler');
            // collisionsHandler.handleReceivedFloatingVote({ block })
        }

        /*

                // broadcast to neighbors
                if (iutils.isInCurrentCycle(block_super.creationDate)) {
                    let pushRes = sendingQ.pushIntoSendingQ({
                        sqType: block_super.bType,
                        sqCode: block_super.blockHash,
                        sqPayload: utils.stringify(block),
                        sqTitle: "Broadcasting the confirmed bVote block(${utils.hash6c(block_super.blockHash)}) ${block_super.cycle}"
                    })
                    clog.app.info("bVote pushRes: ${pushRes}");
                }

                return { err: true, msg, shouldPurgeMessage: true };
            }


                */


        let validate_msg: String = "".to_string();
        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: true,
            m_message: validate_msg,
        };
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

    */

    // old name was getSignMsgBFVote
    pub fn get_sign_msg_b_f_vote(&self, block: &Block) -> String
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
        c_date: &String) -> (bool, Block)
    {
        dlog(
            &format!(
                "Create Floating Vote Block uplink({}) block Cat({}) cDate({}) vote Data: {:?}",
                cutils::hash8c(uplink),
                block_category,
                c_date,
                vote_data
            ),
            constants::Modules::App,
            constants::SecLevel::Info);

        let now_ = application().now();
        let (_backer_address, _shares, percentage) = get_machine_shares(&now_);
        let min_share: SharesPercentT = get_min_share_to_allowed_issue_f_vote(c_date);
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
                    c_date,
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

        let (_status, mut block) = Block::load_block(&j_block);

        // create floating vote block
        let tobe_signed = block.m_if_floating_vote_block.get_sign_msg_b_f_vote(&block);
        let (_status, backer, unlock_set, signatures) =
            machine().sign_by_machine_key(&tobe_signed, 0);
        let mut ext_info = DocExtInfo::new();
        ext_info.m_unlock_set = unlock_set;
        ext_info.m_signatures = vec![signatures];

        block.m_block_ext_info = vec![vec![ext_info]];

        block.m_block_backer = backer;

        let (_status_ext, fv_b_ext_hash) = block.calc_block_ext_root_hash();
        block.m_block_ext_root_hash = fv_b_ext_hash;
        block.set_block_hash(&block.calc_block_hash());

        return (true, block);
    }
}