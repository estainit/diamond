use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{constants, cutils, dlog, machine};
use crate::cutils::remove_quotes;
use crate::lib::block_utils::wrap_safe_content_for_db;
use crate::lib::custom_types::{ClausesT, JSonObject, VString};
use crate::lib::dag::dag::search_in_dag;
use crate::lib::dag::dag_walk_through::getCachedBlocksHashes;
use crate::lib::database::abs_psql::{ModelClause, q_delete, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::{CDEV_PARSING_Q, C_PARSING_Q};

/*

void ParsingQHandler::loopSmartPullFromParsingQ()
{
  String thread_prefix = "pull_from_parsing_q_";
  String thread_code = String::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::RUNNING);
    MissedBlocksHandler::refreshMissedBlock();

    smartPullQ();

    CLog::log("Smart Pull From Parsing Q, Every (" + String::number(CMachine::getParsingQGap()) + " seconds) ", "app", "trace");
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getParsingQGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Smart Pull From Parsing Q");
}

/**
 * @return std::tuple<bool status, bool should_purge_record>
 */
std::tuple<bool, bool> ParsingQHandler::handlePulledPacket(const QVDicT& packet)
{
//  listener.doCallAsync('APSH_before_handle_pulled_packet', args);

  CLog::log("handle Pulled Packet: " + cutils::dumpIt(packet), "app", "trace");

  String receive_date = packet["pq_receive_date"] cutils::get_now()).to_string();
  String pq_type = packet["pq_type"] "").to_string();
  String pq_code = packet["pq_code"] "").to_string();
  String pq_sender = packet["pq_sender"] "").to_string();
  String connection_type = packet["pq_connection_type"] "").to_string();
  /**
  * payload could be a block, GQL or even old-style messages
  * TODO: optimizine to use heap allocation for bigger payloads
  */
  JSonObject payload = packet["pq_payload"] JSonObject()).toJSonObject();

  if ((pq_sender == "") || (payload.keys().len() == 0))
  {
    CLog::log("missed sender or paylod to parse", "app", "error");
    return {false, true};
  }

  if (pq_type == "")
  {
    CLog::log("missed pq_type " + cutils::dumpIt(packet), "app", "error");
    return {false, true};
  }
  if (connection_type == "")
  {
    CLog::log("missed connection_type in parsing " + cutils::dumpIt(packet), "app", "error");
    return {false, true};
  }

  if(payload["bType"].to_string() == constants::BLOCK_TYPES::RpBlock)
  {
    CLog::log("A repay Block received block(" + cutils::hash8c(payload["bHash"].to_string()) + ")", "trx", "info");
    // Since machine must create the repayments by itself we drop this block immidiately,
    // in addition machine calls importCoinbasedUTXOs method to import potentially minted coins and cut the potentially repay backs in on shot
    return {true, true};
  }



  if (StringList {constants::BLOCK_TYPES::Normal,
  constants::BLOCK_TYPES::Coinbase,
  constants::BLOCK_TYPES::FSign,
  constants::BLOCK_TYPES::SusBlock,
  constants::BLOCK_TYPES::FVote,
  constants::BLOCK_TYPES::POW}.contains(pq_type))
  {
    payload["local_receive_date"] = receive_date;
    Block* block = BlockFactory::create(payload);

    if (!block->objectAssignmentsControlls())
    {
      CLog::log("Maleformed JSon block couldn't be parsed! block(" + cutils::hash8c(payload["bHash"].to_string()) + ")", "trx", "error");
      return {false, true};
    }

    auto[status2, should_purge_record2] = parsePureBlock(
      pq_sender,
      pq_type,
      block,
      connection_type,
      receive_date
    );
    delete block;

    if (!CMachine::is_in_sync_process())
      CGUI::refreshMonitor();

    return {status2, should_purge_record2};

  }

  CLog::log(
    "\n\n--- parsing CPacket type(" + pq_type + ") Block/Message \nfrom Q.sender(" + pq_sender + ") ", "app", "trace");

  // GQL part
  if (pq_type == constants::CARD_TYPES::ProposalLoanRequest)
  {
    auto[status, should_purge_record] = GeneralPledgeHandler::handleReceivedProposalLoanRequest(
      pq_sender,
      payload,
      connection_type,
      receive_date);
    if (status)
      CGUI::signalUpdateReceivedLoanRequests();
    return {status, should_purge_record};

  }
  else if (pq_type == constants::CARD_TYPES::FullDAGDownloadRequest)
  {
    auto[status, should_purge_record] = FullDAGHandler::prepareFullDAGDlResponse(
      pq_sender,
      payload,
      connection_type);
    return {status, should_purge_record};

  }
  else if (pq_type == constants::CARD_TYPES::pleaseRemoveMeFromYourNeighbors)
  {
//    case GQLHandler.cardTypes.pleaseRemoveMeFromYourNeighbors:
//        res = require('../../machine/machine-handler').neighborHandler.doDeleteNeighbor({
//            sender,
//            payload,
//            connection_type,
//            receive_date
//        });
//        break;
  }
  else if (pq_type == constants::card_types::DAG_INVOKE_BLOCK)
  {
    //comunications
    auto[status, should_purge_record] = DAGMessageHandler::handleBlockInvokeReq(
      pq_sender,
      payload,
      connection_type);
    return {status, should_purge_record};

  }
  else if (pq_type == constants::card_types::DAG_INVOKE_DESCENDENTS)
  {
//    case message_types.DAG_INVOKE_DESCENDENTS:
//        res = dagMsgHandler.handleDescendentsInvokeReq({
//            sender,
//            payload,
//            connection_type: connection_type
//        })
//        break;

  }

  CLog::log("Unknown packet in parsing Q! " + pq_type + " " + pq_code + " from " + pq_sender, "sec", "error");
  return {false, true};

}






/**
 * @return std::tuple<bool status, bool should_purge_record>
 */
std::tuple<bool, bool> ParsingQHandler::parsePureBlock(
  const String& sender,
  const String& pq_type,
  const Block* block,
  const String& connection_type,
  const String& receive_date
  )
{
  Q_UNUSED(sender);
  Q_UNUSED(connection_type);
  Q_UNUSED(receive_date);

  // DAG existance ancestors controlls
  StringList needed_blocks = cutils::arrayDiff(block.m_ancestors, DAG::getCachedBlocksHashes());
  if (needed_blocks.len() > 0)
  {
    CLog::log(
      "in order to parse 1block(" + cutils::hash6c(block->getBlockHash()) + ") machine needs blocks(" +
      cutils::dumpIt(needed_blocks) + ") exist in DAG"
      "app", "trace");

    // TODO: maybe some reputation system to report diorder of neighbor
    return {false, false};
  }

  auto[b_status, b_should_purge_record] = block->blockGeneralControls();
  if (!b_status)
    return {false, b_should_purge_record};

  // general ancestors controlls
  auto[status, should_purge_record] = ancestorsConroll(pq_type, block);
  if (!status)
    return {status, should_purge_record};


  return block->handleReceivedBlock();

//  switch (pq_type) {

//    case iConsts.BLOCK_TYPES.FVote:
//        res = require('../../dag/floating-vote/floating-vote-handler').handleReceivedFVoteBlock({
//            sender,
//            block,
//            connection_type,
//            receive_date
//        });
//        break;

//    case iConsts.BLOCK_TYPES.POW:
//        res = require('../../dag/pow-block/handle-received-block').handleReceivedPOWBlock({
//            sender,
//            block,
//            connection_type,
//            receive_date
//        });
//        break;
//  }

//  return res;

}

*/


//old_name_was pushToParsingQ
pub fn push_to_parsing_q(
    card_j_obj: &JSonObject,
    creation_date: &String,
    card_type: &String,
    card_code: &String,
    sender: &String,
    connection_type: &String,
    prerequisites: Vec<String>) -> (bool, bool)
{
    let mut prerequisites = prerequisites;
    // check for duplicate entries
    let (status, records) = q_select(
        C_PARSING_Q,
        vec!["pq_type"],
        vec![
            simple_eq_clause("pq_type", card_type),
            simple_eq_clause("pq_code", card_code),
        ],
        vec![],
        0,
        false,
    );
    if records.len() > 0
    { return (true, true); }

    // control if needs some initiative prerequisites
    let mut card_ancestors: VString = vec![];
    if !card_j_obj["ancestors"].is_null()
    {
        if !card_j_obj["ancestors"][0].is_null()
        {
            let mut i = 0;
            while !card_j_obj["ancestors"][i].is_null() {
                card_ancestors.push(card_j_obj["ancestors"][i].to_string());
                i += 1;
            }
        }

//      // check if ancestores exist in parsing q
//      QueryRes queuedAncs = DbModel::select(
//        stbl_parsing_q,
//        {"pq_code"},
//        {{"pq_code", message_ancestors, "IN"}});

//      StringList missedAnc = {};
//      if (queuedAncs.records.len() == 0)
//      {
//        missedAnc = message_ancestors;
//        CLog::log("block(" + code + ") totaly missed ancestors (" + cutils::dumpIt(missedAnc) + ")", "app", "trace");
//      }
//      else if (queuedAncs.records.len() < message_ancestors.len())
//      {
//        StringList pq_codes = {};
//        for(QVDicT a_row: queuedAncs.records)
//          pq_codes.push(a_row["pq_code"].to_string());
//        missedAnc = cutils::arrayDiff(message_ancestors, pq_codes);
//        CLog::log("block(" + code + ") partially missed ancestors (" + cutils::dumpIt(missedAnc) + ") ", "app", "trace");
//      }

        dlog(
            &format!("block({}) before + missed ancs ({:?}) ({:?})", card_code, prerequisites, card_ancestors),
            constants::Modules::App,
            constants::SecLevel::Info);

        // remove if missed anc already exist in cache?
        card_ancestors = cutils::array_diff(&card_ancestors, &getCachedBlocksHashes());

        if card_ancestors.len() > 0
        {
            // remove if missed anc already exist in DAG?
            let daged_blocks = search_in_dag(
                vec![ModelClause {
                    m_field_name: "b_hash",
                    m_field_single_str_value: "",
                    m_clause_operand: "IN",
                    m_field_multi_values: card_ancestors.iter().map(|x| x.as_str()).collect::<Vec<&str>>(),
                }],
                vec!["b_hash"],
                vec![],
                0,
                false,
            );
            if daged_blocks.len() > 0
            {
                card_ancestors = cutils::array_diff(&card_ancestors, &daged_blocks.iter().map(|r, | r["b_hash"].to_string()).collect::<Vec<String>>());
            }
        }

        dlog(
            &format!("Some likely missed blocks({})", card_ancestors.join(",")),
            constants::Modules::App,
            constants::SecLevel::Info);

        prerequisites = cutils::array_add(&prerequisites, &card_ancestors);
    }

    // * if blcok is FVote, maybe we need customized treatment, since generally in DAG later blocks are depend on
    // * early blocks and it is one way graph.
    // * but in case of vote blocks, they have effect on previous blocks (e.g accepting or rejecting a transaction of previously block)
    // * so depends on voting type(bCat) for, we need proper treatment

    if remove_quotes(&card_j_obj["bType"].to_string()) == constants::block_types::FVote
    {
        /*
        if (message["bCat"].to_string() == constants::FLOAT_BLOCKS_CATEGORIES::Trx)
        {
            /**
             * if the machine get an FVote, so insert uplink block in SUS BLOCKS WHICH NEEDED VOTES TO BE IMPORTED AHAED(SusBlockWNVTBIA)
             * WNVTBIA: Wait becaue Needs Vote To Be Importable
             */
            String
            uplinkBlock = message["ancestors"].toArray()[0].to_string();    // FVote blocks always have ONLY one ancestor for which Fvote is voting
            String
            currentWNVTBIA = KVHandler::getValue("SusBlockWNVTBIA");
            StringList
            currentWNVTBIA_arr = {};
            if (currentWNVTBIA == "")
            {
                currentWNVTBIA_arr.push(uplinkBlock);
            } else {
                auto
                tmp = cutils::parseToJsonArr(currentWNVTBIA);
                for (auto x: tmp)
                currentWNVTBIA_arr.push(x.to_string());
                currentWNVTBIA_arr.push(uplinkBlock);
                currentWNVTBIA_arr = cutils::arrayUnique(currentWNVTBIA_arr);
            }
            currentWNVTBIA = cutils::serializeJson(currentWNVTBIA_arr);
            KVHandler::upsertKValue("SusBlockWNVTBIA", currentWNVTBIA);
        }
        */
    }

    // TODO: security issue to control block (specially payload), before insert to db
    // potentially attacks: sql injection, corrupted JSON object ...

    let (_status, _safe_version, pq_payload) = wrap_safe_content_for_db(
        &cutils::serialize_json(&card_j_obj), constants::DEFAULT_SAFE_VERSION);
    let now_ = cutils::get_now();
    let pq_prerequisites = prerequisites.join(",");
    let zero: i32 = 0;
    let pq_v_status = "new".to_string();
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("pq_type", card_type as &(dyn ToSql + Sync)),
        ("pq_code", card_code as &(dyn ToSql + Sync)),
        ("pq_sender", sender as &(dyn ToSql + Sync)),
        ("pq_connection_type", connection_type as &(dyn ToSql + Sync)),
        ("pq_receive_date", &now_ as &(dyn ToSql + Sync)),
        ("pq_payload", &pq_payload as &(dyn ToSql + Sync)),
        ("pq_prerequisites", &pq_prerequisites as &(dyn ToSql + Sync)),
        ("pq_parse_attempts", &zero as &(dyn ToSql + Sync)),
        ("pq_v_status", &pq_v_status as &(dyn ToSql + Sync)),
        ("pq_creation_date", creation_date as &(dyn ToSql + Sync)),
        ("pq_insert_date", &now_ as &(dyn ToSql + Sync)),
        ("pq_last_modified", &now_ as &(dyn ToSql + Sync)),
    ]);

    q_insert(
        C_PARSING_Q,
        &values,
        false);

//    listener.doCallSync('SPSH_after_insert_packet_in_q', args);

    if machine().is_develop_mod()
    {
        q_insert(
            CDEV_PARSING_Q,
            &values,
            false);
    }


    rmoveFromParsingQ(vec![
        ModelClause {
            m_field_name: "pq_parse_attempts",
            m_field_single_str_value: &constants::MAX_PARSE_ATTEMPS_COUNT.to_string(),
            m_clause_operand: ">",
            m_field_multi_values: vec![],
        },
        ModelClause {
            m_field_name: "pq_creation_date",
            m_field_single_str_value: &cutils::minutes_before(cutils::get_cycle_by_minutes(), &cutils::get_now()),
            m_clause_operand: "<",
            m_field_multi_values: vec![],
        }]);

    return (true, true);
}

pub fn rmoveFromParsingQ(clauses: ClausesT) -> bool
{
    return q_delete(
        C_PARSING_Q,
        clauses,
        false,
    );
}
