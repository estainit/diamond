use std::collections::HashMap;
use postgres::types::ToSql;
use serde_json::json;
use crate::{application, constants, cutils, dlog, get_value, machine};
use crate::cutils::remove_quotes;
use crate::lib::custom_types::{CDateT, JSonObject, QVDRecordsT, TimeBySecT};
use crate::lib::dag::dag::search_in_dag;
use crate::lib::dag::leaves_handler::{get_leave_blocks, LeaveBlock};
use crate::lib::dag::missed_blocks_handler::add_missed_blocks_to_invoke;
use crate::lib::database::abs_psql::{q_upsert, simple_eq_clause};
use crate::lib::database::tables::C_KVALUE;
use crate::lib::k_v_handler::{search_in_kv, set_value};
use crate::lib::messaging_protocol::dispatcher::{make_a_packet, PacketParsingResult};
use crate::lib::sending_q_handler::sending_q_handler::push_into_sending_q;

//old_name_was setLastReceivedBlockTimestamp
pub fn set_last_received_block_timestamp(
    block_type: &String,
    block_hash: &String,
    receive_date: &String) -> bool
{
    let kv_value = cutils::serialize_json(&json!({
        "last_block_type": block_type,
        "last_block_hash": block_hash,
        "last_block_receive_date": receive_date
    }));
    let now_ = application().get_now();
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("kv_value", &kv_value as &(dyn ToSql + Sync)),
        ("kv_last_modified", &now_ as &(dyn ToSql + Sync)),
    ]);

    q_upsert(
        C_KVALUE,
        "kv_key",
        "last_received_block_timestamp",
        &values,
        false);
    return true;
}

/*
bool DAGMessageHandler::invokeDescendents(
  const bool &denay_double_send_check)
{
  // read latest recorded block in DAG
  auto[status, block_hash, block_creation_date] = DAG::getLatestBlock();
  Q_UNUSED(status);

  if (cutils::time_diff(block_creation_date).asMinutes > machine().getAcceptableBlocksGap())
  {
    // control if block's potentially descendent(s) exist in parsing q
    QVDRecordsT likeHashRes = ParsingQHandler::searchParsingQ(
      {
        {"pq_type", {constants::BLOCK_TYPES::Normal, constants::block_types::COINBASE}, "IN"},
        {"pq_code", block_hash}
      },
      {"pq_type", "pq_code", "pq_payload"});

    // invoke network for block probably descendents
    StringList existed_descendents_in_parsingQ = {};
    if (likeHashRes.len() > 0)
    {
      for (QVDicT wBlock: likeHashRes)
      {
        JSonObject jBlock = cutils::parseToJsonObj(wBlock["pq_payload"].to_string());
        // if the existed block in parsing q is descendent of block
        StringList tmp = {};
        for(QJsonValueRef an_anc: jBlock["ancestors"].toArray())
          tmp.push(an_anc.to_string());
        if (tmp.contains(block_hash))
          existed_descendents_in_parsingQ.push(jBlock["bHash"].to_string());
      }
    }
    if (existed_descendents_in_parsingQ.len() > 0)
    {
      // controling if the ancestors of descendent exist in local or not
      existed_descendents_in_parsingQ = cutils::arrayUnique(existed_descendents_in_parsingQ);
      return blockInvokingNeeds(existed_descendents_in_parsingQ);
      // set prerequisities null and attemps zero in order to force machine parsing them

    } else {
      // Machine doesn't know about block descendents, so asks network
      return doInvokeDescendents(
        block_hash,
        block_creation_date,
        denay_double_send_check);

    }
  }
  return false;
}

bool DAGMessageHandler::doInvokeDescendents(
  const String &block_hash,
  const String &block_creation_date,
  const bool &denay_double_send_check)
{
  CLog::log("do Invoke Descendents args block_hash(" + block_hash + ") block_creation_date(" + block_creation_date + ") denay_double_send_check(" + cutils::dumpIt(denay_double_send_check) + ")", "app", "trace");

  // if the last block which exists in DAG is older than 2 cycle time maybe efficient to call full-history
  if (block_creation_date < cutils::minutes_before(2 * cutils::get_cycle_by_minutes()))
  {
    String LastFullDAGDownloadResponse = KVHandler::getValue("LAST_FULL_DAG_DOWNLOAD_RESPONSE");
    if (LastFullDAGDownloadResponse == "")
    {
      KVHandler::upsertKValue("LAST_FULL_DAG_DOWNLOAD_RESPONSE", cutils::minutes_before(cutils::get_cycle_by_minutes()));

    } else {
      if (cutils::time_diff(LastFullDAGDownloadResponse).asMinutes < 5)
      {
        CLog::log("less than 5 minutes ago invoked for full DAG", "app", "trace");
        return true;
      }
    }

    // TODO: improve it to not send full req to all neighbors
    FullDAGHandler::invokeFullDAGDlRequest(block_creation_date);


  } else {

    CLog::log("invoking for descendents of ${utils.hash6c(block_hash)}", "app", "trace");
    JSonObject payload = {
      {"mType", constants::card_types::DAG_INVOKE_DESCENDENTS},
      {"mVer", "0.0.0"},
      {"bHash", block_hash}
    };
    String payload_ = cutils::serializeJson(payload);
    SendingQHandler::pushIntoSendingQ(
      constants::card_types::DAG_INVOKE_DESCENDENTS,
      block_hash, // sqCode
      payload_,
      "Invoke Descendents(" + cutils::hash16c(block_hash) + ")",
      {},
      {},
      denay_double_send_check);
  }

  return true;
}

/**
* the method (going back in history) analyzes block(s) prerequisities and maybe invoke them
* @param {*} block_hash
* @param {*} level
*/
bool DAGMessageHandler::blockInvokingNeeds(
  StringList block_hashes,
  uint level)
{


  StringList next_level_block_hashes = {};
  StringList missed_blocks = {};
  for (uint l = 0; l < level; l++)
  {
    // exists in DAG?
    QVDRecordsT existedInDAG = DAG::searchInDAG(
      {{"b_hash", block_hashes, "IN"}},
      {"b_hash"});
    if (existedInDAG.len() == block_hashes.len())
      continue; // all blocks are already recorded in local graph

    StringList tmp;
    for(QVDicT a_row: existedInDAG)
      tmp.push(a_row["b_hash"].to_string());
    StringList array_diff = cutils::arrayDiff(block_hashes, tmp);

    // control if block exist in parsing_q
    for (auto looking_hash: array_diff)
    {
      QVDRecordsT existsInParsingQ = ParsingQHandler::searchParsingQ(
        {{"pq_code", looking_hash}},
        {"pq_code", "pq_payload"});

      if (existsInParsingQ.len() == 0)
      {
        missed_blocks.push(looking_hash);
      } else {
//        let ancestors = existsInParsingQ.map(x => JSON.parse(x.pqPayload).ancestors);
        QList<StringList> ancestors;
        for(auto x: existsInParsingQ)
        {
          JSonObject payloadJs = cutils::parseToJsonObj(x["pq_payload"].to_string());
          JSonArray ancsJS = payloadJs["ancestors"].toArray();
          StringList ancestors;
          for(auto y: ancsJS)
            ancestors.push(y.to_string());
        }

        if (ancestors.len() == 0)
        {
          CLog::log("The block(" + cutils::hash16c(looking_hash) + ") has no valid ancestors! " + cutils::dumpIt(existsInParsingQ), "sec", "error");
          return false;
        }
        for(auto pckedAncestors: ancestors)
        {
          for(auto ancestor: pckedAncestors)
          {
            next_level_block_hashes.push(ancestor);
          }
        }
      }
    }
    block_hashes = cutils::arrayUnique(next_level_block_hashes);
  }
  missed_blocks = cutils::arrayUnique(missed_blocks);
  addMissedBlocksToInvoke(missed_blocks);
//  loopMissedBlocksInvoker();
  return true;
}

void DAGMessageHandler::loopMissedBlocksInvoker()
{
  String thread_prefix = "missed_blocks_invoker_";
  String thread_code = String::number((quint64)QThread::currentThread(), 16);

  while (machine().shouldLoopThreads())
  {
    machine().reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::RUNNING);
    doMissedBlocksInvoker();

    machine().reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(machine().getBlockInvokeGap()));
  }

  machine().reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Missed Blocks Invoker");
}
*/

//old_name_was doMissedBlocksInvoker
pub fn do_missed_blocks_invoker()
{
    /*
  String cycle = cutils::getCoinbaseCycleStamp();
  CLog::log("ReMiBcInv cycle(" + cycle + ") called recursive MissedBlocks Invoker", "app", "trace");
  StringList missed = getMissedBlocksToInvoke(2);
  // listener.doCallAsync("APSH_control_if_missed_block");

  if (missed.len() > 0)
  {
    CLog::log("ReMiBcInv cycle(" + cycle + ") recursive Missed Blocks Invoker has " + String::number(missed.len()) + " missed blocks(" + cutils::dumpIt(missed) + ")", "app", "trace");
    for(String a_missed: missed)
    {
      //check if not already exist in parsing q
      QVDRecordsT existsInParsingQ = ParsingQHandler::searchParsingQ(
        {{"pq_code", a_missed}},
        {"pq_type", "pq_code"});

      if (existsInParsingQ.len() == 0)
      {
        invokeBlock(a_missed);
        MissedBlocksHandler::increaseAttempNumber(a_missed);
      }
    }
  }
    */
}
/*
bool DAGMessageHandler::invokeBlock(const String &block_hash)
{
  CLog::log("invoking for block(" + cutils::hash16c(block_hash) + ")", "app", "trace");
  JSonObject payload {
    {"mType", constants::card_types::DAG_INVOKE_BLOCK},
    {"mVer", "0.0.0"},
    {"bHash", block_hash}};
  String serialized_payload = cutils::serializeJson(payload);
  CLog::log("invoked for keaves (" + block_hash + ")", "app", "trace");

  bool status = SendingQHandler::pushIntoSendingQ(
    constants::card_types::DAG_INVOKE_BLOCK, // sqType
    block_hash,  // sqCode
    serialized_payload,
    "Invoke Block(" + cutils::hash16c(block_hash) + ")");

  return status;
}

*/
//old_name_was getLastReceivedBlockTimestamp
pub fn get_last_received_block_timestamp() -> JSonObject
{
    let res: String = get_value("last_received_block_timestamp");
    let now_ = application().launch_date();
    if res == "" {
        return json!({
            "last_block_type": "Genesis",
            "last_block_hash": "-" ,
            "last_block_receive_date": now_});
    }
    return cutils::parse_to_json_obj(&res);
}

//old_name_was getMaybeAskForLatestBlocksFlag
pub fn get_maybe_ask_for_latest_blocks_flag() -> String
{
    return get_value("maybe_ask_for_latest_blocks");
}

//old_name_was invokeLeaves
pub fn invoke_leaves() -> bool
{
    dlog(
        &format!("Invoking for DAG leaves!"),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (code, body) = make_a_packet(
        vec![json!({
            "cdType": constants::card_types::DAG_INVOKE_LEAVES,//CConsts::CARD_TYPES::FullDAGDownloadRequest
            "cdVer": constants::DEFAULT_CARD_VERSION})],
        constants::DEFAULT_PACKET_TYPE,
        constants::DEFAULT_PACKET_VERSION,
        application().get_now(),
    );
    // let payload: JSonObject = json!({
    //     "mType": constants::card_types::DAG_INVOKE_LEAVES,
    //     "mVer": "0.0.0"});
    // let serialized_payload = cutils::serialize_json(&payload);

    let status = push_into_sending_q(
        constants::card_types::DAG_INVOKE_LEAVES, // sqType
        &cutils::hash6c(&code),  // sqCode
        &body,
        "Invoking for DAG leaves",
        &vec![],
        &vec![],
        false,
    );

    return status;
}

//old_name_was launchInvokeLeaves
pub fn launch_invoke_leaves()
{
    let should_i = get_maybe_ask_for_latest_blocks_flag();
    if should_i == constants::YES
    {
        // TODO: needs control for latest invoke to not spaming network
        invoke_leaves();
        /*
        setMaybeAskForLatestBlocksFlag(constants::NO);
        */
    }
}

//old_name_was setMaybeAskForLatestBlocksFlag
pub fn set_maybe_ask_for_latest_blocks_flag(value: &str)
{
    dlog(
        &format!("set Maybe Ask For Latest Blocks Flag value: {}", value),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    if value == constants::YES {
        // control last_received_leaves_info_timestamp flag
        // if we currently asked for leave information, so do not flood the network with multiple asking
        let last_leave_invoke_response_str: String = get_value("last_received_leaves_info_timestamp");
        if last_leave_invoke_response_str != "" {
            let last_leave_invoke_response: JSonObject = cutils::parse_to_json_obj(&last_leave_invoke_response_str);
            // TODO: tune the gap time
            let now_ = application().get_now();
            if application().time_diff(
                last_leave_invoke_response["receiveDate"].to_string(),
                now_).as_seconds
                < machine().get_invoke_leaves_gap() {
                return;
            }
        }

        // control last_received_block_timestamp flag
        // if we are receiving continiuosly new blocks, it doesn't sence to ask for leave information.
        // this case happends in runing a new machin in which the machine has to download entire DAG.
        let last_block: JSonObject = get_last_received_block_timestamp();

        // TODO: tune the gap time
        let now_ = application().get_now();
        let invoke_gap = application().time_diff(
            remove_quotes(&last_block["last_block_receive_date"]),
            now_).as_seconds;

        let minimum_leave_invoke_gap = machine().get_invoke_leaves_gap();
        if invoke_gap < minimum_leave_invoke_gap
        {
            dlog(
                &format!("Can not invoke leaves because just passed ({}) less than required gap ({})", invoke_gap, minimum_leave_invoke_gap),
                constants::Modules::App,
                constants::SecLevel::Debug);
            return;
        }

        let machine_request_status: QVDRecordsT = search_in_kv(
            vec![simple_eq_clause("kv_key", &"maybe_ask_for_latest_blocks".to_string())],
            vec!["kv_last_modified"],
            vec![],
            0);
        if machine_request_status.len() > 0 {
            let now_ = application().get_now();
            let invoke_age: TimeBySecT = application().time_diff(
                machine_request_status[0]["kv_last_modified"].to_string(),
                now_).as_seconds;
            dlog(
                &format!("control if (invoke_age: {} < (invokeGap: {}) ", invoke_age, minimum_leave_invoke_gap),
                constants::Modules::App,
                constants::SecLevel::Info);
            if invoke_age < minimum_leave_invoke_gap {
                return;
            }
        }

        // TODO: tune the gap time
        launch_invoke_leaves();
    }

    set_value("maybe_ask_for_latest_blocks", value, false);
}

//old_name_was extractLeavesAndPushInSendingQ
pub fn extract_leaves_and_push_in_sending_q(sender: &String) -> PacketParsingResult
{
    let leaves: HashMap<String, LeaveBlock> = get_leave_blocks(&"".to_string());
    let mut new_leaves: Vec<JSonObject> = vec![];
    for (_k, v) in leaves
    {
        new_leaves.push(json!({
            "bType": v.m_block_type,
            "bHash": v.m_block_hash,
            "cDate": v.m_creation_date
        }));
    }
    dlog(
        &format!("leaves in DAG: {:?}", new_leaves),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (code, body) = make_a_packet(
        vec![
            json!({
                "cdType": constants::card_types::DAG_LEAVES_INFO,
                "cdVer": constants::DEFAULT_CARD_VERSION,
                "leaves": new_leaves
            }),
        ],
        constants::DEFAULT_PACKET_TYPE,
        constants::DEFAULT_PACKET_VERSION,
        application().get_now(),
    );
    dlog(
        &format!("prepared packet, before insert into DB code({}) to ({}): {}", code, sender, body),
        constants::Modules::App,
        constants::SecLevel::Info);

    let status = push_into_sending_q(
        constants::card_types::DAG_LEAVES_INFO,
        &code,
        &body,
        &format!("DAG info response for {}", sender),
        &vec![sender.to_string()],
        &vec![],
        false,
    );

    return PacketParsingResult{
        m_status: true,
        m_should_purge_file: status,
        m_message: "".to_string()
    };
}

/*

std::tuple<bool, bool> DAGMessageHandler::handleBlockInvokeReq(
  const String& sender,
  const JSonObject& payload,
  const String& connection_type)
{
  const CBlockHashT& block_hash = payload["bHash"].to_string();

  String short_hash = cutils::hash8c(block_hash);
  CLog::log("handle Block Invoke Req block(" + short_hash + ")", "app", "trace");

  // retrieve block from DAG
  auto[status, regenerated_json_block] = Block::regenerateBlock(block_hash);

  if (!status)
  {
    // TODO: the block is valid and does not exist in local. or
    // invalid block invoked, maybe some penal for sender!
    // msg = `The block (${short}) invoked by ${args.sender} does not exist in local. `;
    // clog.sec.error(msg);
    CLog::log("Invoked block regenration failed! Block(" + short_hash + ")", "app", "error");
    return {false, true};
  }

  CLog::log("Broadcasting Replay to invoke for block(" + regenerated_json_block["bType"].to_string() + " / " + cutils::hash8c(block_hash) + ")", "app", "trace");
  Block* block = BlockFactory::create(regenerated_json_block);
  bool push_res = SendingQHandler::pushIntoSendingQ(
    block.m_block_type,
    block_hash,
    block->safe_stringify_block(false),
    "Replay to invoke for block(" + short_hash + ") type(" + block.m_block_type  + ")",
    {sender});

  CLog::log("invoke block push_res(" + cutils::dumpIt(push_res) + ") block(" + short_hash + ") type(" + block.m_block_type  + ")", "app", "trace");

  return {true, true};
}

 */

//old_name_was handleReceivedLeaveInfo
pub fn handle_received_leave_info(
    _sender_email: &String,
    message: &JSonObject,
    _connection_type: &String) -> PacketParsingResult
{
    dlog(
        &format!("FIX ME: What part of message must be recorded in db? {:?}", message),
        constants::Modules::App,
        constants::SecLevel::Error);

    let leaves: Vec<JSonObject> = vec![]; // = message.clone();
    // update last_received_leaves_info_timestamp
    set_last_received_leave_info_timestamp(&leaves, &application().get_now());

    // control if block exist in local, if not adding to missed blocks to invoke
    let mut missed_hashes: Vec<String> = vec![];
    for a_leave in &leaves
    {
        let a_leave_hash = a_leave["bHash"].to_string();
        let already_recorded_in_dag = search_in_dag(
            vec![simple_eq_clause("b_hash", &a_leave_hash)],
            vec!["b_hash"],
            vec![],
            0,
            false);
        if already_recorded_in_dag.len() == 0
        {
            missed_hashes.push(a_leave_hash);
        }
    }

    add_missed_blocks_to_invoke(missed_hashes);

    //maybe launch missed block invoker
    //launchMissedBlocksInvoker()   // FIXME: do it in Async mode or thread


        return PacketParsingResult {
            m_status: true,
            m_should_purge_file: true,
            m_message: "".to_string(),
        };
}

//old_name_was setLastReceivedLeaveInfoTimestamp
pub fn set_last_received_leave_info_timestamp(leaves: &Vec<JSonObject>, c_date: &CDateT)
{
    let last_modified = application().get_now();
    let kv_value = cutils::serialize_json(&json!({
        "leaves": leaves,
        "receiveDate": c_date
    }));
    let update_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("kv_value", &kv_value as &(dyn ToSql + Sync)),
        ("kv_last_modified", &last_modified as &(dyn ToSql + Sync)),
    ]);

    q_upsert(
        C_KVALUE,
        "kv_key",
        "last_received_leaves_info_timestamp",
        &update_values,
        false,
    );
}
