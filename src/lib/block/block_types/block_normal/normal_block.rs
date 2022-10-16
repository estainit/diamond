use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::{application, ccrypto, constants, cutils, dlog, machine};
use crate::lib::block::block_types::block::{Block, TransientBlockInfo};
use crate::lib::block::block_types::block_floating_vote::floating_vote_block::FloatingVoteBlock;
use crate::lib::block::documents_in_related_block::transactions::transactions_in_related_block::validate_transactions;
use crate::lib::custom_types::{CBlockHashT, VString};
use crate::lib::dag::dag::is_dag_updated;
use crate::lib::messaging_protocol::dispatcher::make_a_packet;
use crate::lib::parsing_q_handler::queue_pars::EntryParsingResult;
use crate::lib::sending_q_handler::sending_q_handler::push_into_sending_q;
use crate::lib::transactions::basic_transactions::coins::coins_handler::remove_used_coins_by_block;
use crate::lib::transactions::basic_transactions::coins::spent_coins_handler::{SpendCoinsList, SpentCoinsHandler};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NormalBlock
{}

impl NormalBlock
{
    pub fn new() -> Self
    {
        Self {}
    }

    pub fn calc_block_hash(&self, block: &Block) -> CBlockHashT
    {
        let block_hash_ables: String = self.get_block_hashable_string(block);
        // clonedTransactionsRootHash: block.clonedTransactionsRootHash, // note that we do not put the clonedTransactions directly in block hash, instead using clonedTransactions-merkle-root-hash

        dlog(
            &format!("The NORMAL block hashable: {}", block_hash_ables),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return ccrypto::keccak256(&block_hash_ables);
    }

    //old_name_was getBlockHashableString
    pub fn get_block_hashable_string(&self, block: &Block) -> String
    {
        // in order to have almost same hash! we sort the attributes alphabetically
        let block_hash_ables: String = format!(
            "bAncestors:{},bBacker:{},bCDate:{},bDocsRootHash:{},bExtHash:{},bLen:{},bNet:{},bSignals:{},bType:{},bVer:{},bFVotes:{}",
            serde_json::to_string(&block.m_block_ancestors).unwrap(),
            block.m_block_backer,
            block.m_block_creation_date,
            block.m_block_documents_root_hash, // note that we do not put the docsHash directly in block hash, instead using docsHash-merkle-root-hash
            block.m_block_ext_root_hash,    // note that we do not put the segwits directly in block hash, instead using segwits-merkle-root-hash
            block.m_block_length,
            block.m_block_net,
            serde_json::to_string(&block.m_block_signals).unwrap(),
            block.m_block_type,
            block.m_block_version,
            self.stringify_floating_votes()
        );
        return block_hash_ables;
    }

    // old name was stringifyFloatingVotes
    pub fn stringify_floating_votes(&self) -> String
    {
        return "[]".to_string();
        // // process m_floating_votes (if exist)
        // // legacy including unimplemented feature in blocks in order to forward compatibility
        // let  mut      fVotes:Vec<JSonObject>=vec![];
        // if self.m_block_floating_votes.len() > 0
        // {
        //     fVotes = self.m_block_floating_votes;
        // }
        // return cutils::serializeJson(fVotes);
    }

    // old name was validateNormalBlock
    pub fn validate_normal_block
    (&self, block_super: &Block, stage: &str)
     -> (
         bool, // status
         bool, // is_sus_block
         String, // message
         SpendCoinsList // double_spends
     )
    {
        let mut msg: String = "".to_string();
        dlog(
            &format!(
                "xxxxxxxxxxxx validate Normal Block xxxxxxxxxxxxxxxxxxxx{}, {:?}",
                block_super.get_block_identifier(),
                block_super),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        let (status, is_sus_block, validate_msg, double_spends)
            = validate_transactions(block_super, stage);
        if !status
        {
            return (false, false, validate_msg, SpendCoinsList::new());
        }
        dlog(
            &format!(
                "validate-transactions, done, {}",
                block_super.get_block_identifier()),
            constants::Modules::App,
            constants::SecLevel::Debug);

        let transient_block_info: TransientBlockInfo = block_super.group_docs_of_block(stage);
        if !transient_block_info.m_status
        {
            msg = format!("Failed in group-Docs-Of-Block {}", block_super.get_block_identifier());
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, false, msg, SpendCoinsList::new());
            // grpdRes.shouldPurgeMessage = true;
        }
        dlog(
            &format!(
                "group-docs-of-block, done, {}",
                block_super.get_block_identifier()),
            constants::Modules::App,
            constants::SecLevel::Debug);

        let mut dTyps: VString = transient_block_info.m_grouped_documents.keys().cloned().collect();
        dTyps.sort();
        dlog(
            &format!(
                "Block{} contains these doc types: {:?}",
                block_super.get_block_identifier(),
                dTyps),
            constants::Modules::App,
            constants::SecLevel::Debug);

        // control if each trx is referenced to only one Document?
        let mut tmp_trxs: VString = vec![];
        for key in transient_block_info.m_map_trx_ref_to_trx_hash.keys().cloned().collect::<VString>()
        {
            tmp_trxs.push(transient_block_info.m_map_trx_ref_to_trx_hash[&key].clone());
        }
        if tmp_trxs.len() != cutils::array_unique(&tmp_trxs).len()
        {
            msg = format!(
                "Invalid block! same transaction is used as a ref for different docs! Block{} mapTrxRefToTrxHash({:?})",
                block_super.get_block_identifier(),
                transient_block_info.m_map_trx_ref_to_trx_hash);
            dlog(
                &msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return (false, false, msg, SpendCoinsList::new());
        }
        dlog(
            &format!(
                "map-trx-ref-to-trx-hash, done, {}",
                block_super.get_block_identifier()),
            constants::Modules::App,
            constants::SecLevel::Debug);


        // TODO: important! currently the order of validating documents of block is important(e.g. polling must be validate before proposals and pledges)
        // improve the code and remove this dependency

        //  // * validate polling request(if exist)
        // let status_polling = PollingsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
        // if (!status_polling)
        //   return {false, true, "Failed in validate-In-Block polling", {}};

        //  // * validate requests for administrative polling(if exist)
        // bool adm_polling_validate_res = AdministrativePollingsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
        // if (!adm_polling_validate_res)
        //   return {false, false, "Failed in validate-In-Block adm-polling", {}};

        //  /**
        //   * validate reqRelRes request(if exist)
        //   * TODO: move it to validate
        //   */
        //  let reserveCoinsValidateRes = reqRelRessInRelatedBlock.validateReqRelRess(validateParams);
        //  if (reserveCoinsValidateRes.err != false) {
        //      return reserveCoinsValidateRes;
        //  }

        //  * validate vote-ballots (if exist)
        // bool ballots_validate_res = BallotsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
        // if (!ballots_validate_res)
        //   return {false, false, "Failed in validate-In-Block votes", {}};

        //  * validate proposals (if exist)
        // bool proposals_validate_res = ProposalsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
        // if (!proposals_validate_res)
        //   return {false, false, "Failed in validate-In-Block proposals", {}};

        //  * validate pledges (if exist)
        // bool pledges_validate_res = PledgesInRelatedBlock::validateInBlock(this, transient_block_info, stage);
        // if (!pledges_validate_res)
        //   return {false, false, "Failed in validate-In-Block pledges", {}};
        //
        //  * validate close pledges (if exist)
        // bool close_pledges_validate_res = ClosePledgesInRelatedBlock::validateInBlock(this, transient_block_info, stage);
        // if (!close_pledges_validate_res)
        //   return {false, false, "Failed in validate-In-Block close-pledges", {}};
        //

        //  * validate iNames (if exist)
        // bool inames_validate_res = INamesInRelatedBlock::validateInBlock(this, transient_block_info, stage);
        // if (!inames_validate_res)
        //   return {false, false, "Failed in validate-In-Block iNames", {}};
        //
        //  * validate bind-iNames (if exist)
        // bool inames_bindings_validate_res = INamesBindsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
        // if (!inames_bindings_validate_res)
        //   return {false, false, "Failed in validate-In-Block iName bindings", {}};

        //  /**
        //   * validate msg-to-iNames (if exist)
        //   */
        //  let iNameMsgsValidateRes = iNameMsgsInRelatedBlock.validateINameMsgs(validateParams);
        //  if (iNameMsgsValidateRes.err != false) {
        //      return iNameMsgsValidateRes;
        //  }

        //  * validate free-docs (if exist)
        // bool free_documents_validate_res = FreeDocumentsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
        // if (!free_documents_validate_res)
        //   return {false, false, "Failed in validate-In-Block free-docs", {}};

        // validate...

        dlog(
            &format!(
                "--- Confirmed normal block, done, {}",
                block_super.get_block_identifier()),
            constants::Modules::App,
            constants::SecLevel::Info);

        return (
            true,
            is_sus_block,
            "valid".to_string(),
            double_spends
        );
    }


    // js name was handleReceivedNormalBlock
    // old name was handleReceivedBlock
    pub fn handle_received_block(&self, block_super: &Block) -> EntryParsingResult
    {
        let mut msg: String;
        dlog(
            &format!("******** handle Received Normal Block{}", block_super.get_block_identifier()),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        let (status, is_sus_block, validate_msg, double_spends)
            = self.validate_normal_block(block_super, constants::stages::VALIDATING);

        dlog(
            &format!(
                "Received a block {}, validation result: is_sus_block({}) double_spends({:?})",
                block_super.get_block_identifier(),
                is_sus_block,
                double_spends),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        if !status
        {
            dlog(
                &format!("{}", validate_msg),
                constants::Modules::App,
                constants::SecLevel::Error);

            // maybe do something more! e.g. calling reputation system hooks via zmq
            return EntryParsingResult {
                m_status: false,
                m_should_purge_record: true,
                m_message: validate_msg,
            };
        }

        //TODO: prepare a mega query to run in atomic transactional mode
        block_super.add_block_to_dag();
        block_super.post_add_block_to_dag();

        // remove used UTXOs
        remove_used_coins_by_block(block_super);

        // log spend coins
        let mut c_date = application().now();
        // if machine is in sync mode, we send half a cycle after creationdate to avoid deleting all spend records in table "trx_spend"
        if machine().is_in_sync_process(false)
        {
            c_date = block_super.m_block_creation_date.clone();
        }
        SpentCoinsHandler::mark_as_spent_all_block_inputs(block_super, &c_date);

        // broadcast block to neighbors
        if is_dag_updated(&"".to_string())
        {
            let mut block_body = block_super.safe_stringify_block(false);
            dlog(
                &format!(
                    "About to sending a confirmed normal block to network block: {} {}",
                    block_super.get_block_identifier(),
                    block_body),
                constants::Modules::App,
                constants::SecLevel::TmpDebug);

            block_body = ccrypto::b64_encode(&block_body);
            let (_code, body) = make_a_packet(
                vec![
                    json!({
                "cdType": block_super.get_block_type(),
                "cdVer": constants::DEFAULT_CARD_VERSION,
                "bHash": block_super.get_block_hash(),
                // "ancestors": ancestors,
                "block": block_body,
            }),
                ],
                constants::DEFAULT_PACKET_TYPE,
                constants::DEFAULT_PACKET_VERSION,
                application().now(),
            );
            dlog(
                &format!(
                    "prepared confirmed normal block packet, before insert into DB {}: {}",
                    block_super.get_block_identifier(),
                    body),
                constants::Modules::App,
                constants::SecLevel::Info);

            let status = push_into_sending_q(
                &block_super.get_block_type(),
                &block_super.get_block_hash(),
                &body,
                &format!(
                    "Broadcasting confirmed normal block {} {}",
                    block_super.get_block_identifier(), application().now()
                ),
                &vec![],
                &vec![],
                false,
            );

            dlog(
                &format!(
                    "Normal block status {} {}",
                    status,
                    block_super.get_block_identifier()),
                constants::Modules::App,
                constants::SecLevel::Info);

            if is_sus_block
            {
                let (status_sus, tmp_fv_block) = FloatingVoteBlock::create_floating_vote_block(
                    &block_super.get_block_hash(),  // uplink
                    &constants::float_blocks_categories::TRANSACTION.to_string(),  // bCat
                    &SpentCoinsHandler::convert_spends_to_json_object(&double_spends), // voteData
                    &c_date);

                if !status_sus
                {
                    msg = format!(
                        "Failed on generating floating vote(susVote): of block uplink({})",
                        block_super.get_block_identifier());
                    dlog(
                        &msg,
                        constants::Modules::App,
                        constants::SecLevel::Error);
                    return EntryParsingResult {
                        m_status: false,
                        m_should_purge_record: true,
                        m_message: msg,
                    };
                }

                drop(&double_spends);
                let mut block_body = tmp_fv_block.safe_stringify_block(false);
                dlog(
                    &format!(
                        "Broadcasting floating vote(susVote) because of block uplink{}  FVBlock{} {}",
                        block_super.get_block_identifier(),
                        tmp_fv_block.get_block_identifier(),
                        block_body,
                    ),
                    constants::Modules::App,
                    constants::SecLevel::Info);
                block_body = ccrypto::b64_encode(&block_body);
                let (_code, body) = make_a_packet(
                    vec![
                        json!({
                            "cdType": tmp_fv_block.get_block_type(),
                            "cdVer": constants::DEFAULT_CARD_VERSION,
                            "bHash": tmp_fv_block.get_block_hash(),
                            // "ancestors": ancestors,
                            "block": block_body,
                        }),
                    ],
                    constants::DEFAULT_PACKET_TYPE,
                    constants::DEFAULT_PACKET_VERSION,
                    application().now(),
                );
                let status = push_into_sending_q(
                    &tmp_fv_block.get_block_type(),
                    &tmp_fv_block.get_block_hash(),
                    &body,
                    &format!(
                        "Broadcasting floating vote on sus-block{} {} {}",
                        block_super.get_block_identifier(),
                        tmp_fv_block.get_block_identifier(),
                        application().now()
                    ),
                    &vec![],
                    &vec![],
                    false,
                );

                dlog(
                    &format!("Normal block status: {}", status),
                    constants::Modules::App,
                    constants::SecLevel::Info);

                drop(&tmp_fv_block);
            }

            return EntryParsingResult {
                m_status: true,
                m_should_purge_record: true,
                m_message: "Done 1".to_string(),
            };
        } else if machine().is_in_sync_process(false)
        {
            if is_sus_block
            {
                dlog(
                    &format!("Machine in sync mode and found a sus block uplink:{} ", block_super.get_block_identifier()),
                    constants::Modules::App,
                    constants::SecLevel::Info);
                drop(&double_spends);
            }
            return EntryParsingResult {
                m_status: true,
                m_should_purge_record: true,
                m_message: "Done 2".to_string(),
            };
        }
        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: false,
            m_message: "False!".to_string(),
        };
    }



    /*


    bool NormalBlock::setByJsonObj(const JSonObject& obj)
    {
      Block::setByJsonObj(obj);


      // custom settings for Normal block
      auto[shares_, percentage] = DNAHandler::getAnAddressShares(m_block_backer, m_block_creation_date);
      Q_UNUSED(shares_);
      m_block_confidence = percentage;

      return true;
    }

    String NormalBlock::dumpBlock() const
    {
      // firsdt call parent dump
      String out = Block::dumpBlock();

      // then child dumpping
      out += "\n in child";
      return out;
    }

    JSonObject NormalBlock::exportBlockToJSon(const bool ext_info_in_document) const
    {
      JSonObject Jblock = Block::exportBlockToJSon(ext_info_in_document);

      Jblock["fVotes"] = QJsonArray{};  // legacy including unimplemented feaure in blocks in order to forward compatibility
      if (m_floating_votes.len() > 0)
        Jblock["fVotes"] = m_floating_votes;

      Jblock["bLen"] = cutils::paddingLengthValue(calcBlockLength(Jblock));

      return Jblock;
    }

    BlockLenT NormalBlock::calcBlockLength(const JSonObject& block_obj) const
    {
      return Block::calcBlockLength(block_obj);
    }

    bool NormalBlock::controlBlockLength() const
    {
      String stringyfied_block = safeStringifyBlock(false);
      if (static_cast<BlockLenT>(stringyfied_block.len()) != m_block_length)
      {
        CLog::log("Mismatch Normal block length Block(" + cutils::hash8c(m_block_hash) + ") local length(" + String::number(stringyfied_block.len()) + ") remote length(" + String::number(m_block_length) + ") stringyfied_block:" + stringyfied_block, "sec", "error");
        return false;
      }
      return true;
    }

    //QJsonArray NormalBlock::getBlockExtInfoByDocIndex(const CDocIndexT& document_index) const
    //{
    //  return m_block_ext_info.toJsonArray()[document_index].toVariant();
    //}

    */
}