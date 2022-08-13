/*
#include "stable.h"
#include "lib/sending_q_handler/sending_q_handler.h"
#include "lib/parsing_q_handler/parsing_q_handler.h"
#include "lib/dag/full_dag_handler.h"
#include "dag_message_handler.h"

const String DAGMessageHandler::STBL_KVALUE = "c_kvalue";

DAGMessageHandler::DAGMessageHandler()
{

}

bool DAGMessageHandler::setLastReceivedBlockTimestamp(
  const String &bType,
  const String &block_hash,
  const String &receive_date)
{
  DbModel::upsert(
    STBL_KVALUE,
    "kv_key",
    "LAST_RECEIVED_BLOCK_TIMESTAMP",
    {
      {"kv_value", cutils::serializeJson({
        {"last_block_type", bType},
        {"last_block_hash", block_hash},
        {"last_block_receive_date", receive_date}
      })},
      {"kv_last_modified", cutils::get_now()}
    });
  return true;
}

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
        {"pq_type", {constants::BLOCK_TYPES::Normal, constants::BLOCK_TYPES::Coinbase}, "IN"},
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
      {"type", constants::MESSAGE_TYPES::DAG_INVOKE_DESCENDENTS},
      {"mVer", "0.0.0"},
      {"bHash", block_hash}
    };
    String payload_ = cutils::serializeJson(payload);
    SendingQHandler::pushIntoSendingQ(
      constants::MESSAGE_TYPES::DAG_INVOKE_DESCENDENTS,
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
  MissedBlocksHandler::addMissedBlocksToInvoke(missed_blocks);
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
use serde_json::json;
use crate::{constants, cutils, dlog, get_value, machine};
use crate::lib::custom_types::{JSonObject, QVDRecordsT, TimeBySecT};
use crate::lib::database::abs_psql::simple_eq_clause;
use crate::lib::k_v_handler::{search_in_kv, set_value};

//old_name_was doMissedBlocksInvoker
pub fn do_missed_blocks_invoker()
{
    /*
  String cycle = cutils::getCoinbaseCycleStamp();
  CLog::log("ReMiBcInv cycle(" + cycle + ") called recursive MissedBlocks Invoker", "app", "trace");
  StringList missed = getMissedBlocksToInvoke(2);
  // listener.doCallAsync('APSH_control_if_missed_block');

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
    {"type", constants::MESSAGE_TYPES::DAG_INVOKE_BLOCK},
    {"mVer", "0.0.0"},
    {"bHash", block_hash}};
  String serialized_payload = cutils::serializeJson(payload);
  CLog::log("invoked for keaves (" + block_hash + ")", "app", "trace");

  bool status = SendingQHandler::pushIntoSendingQ(
    constants::MESSAGE_TYPES::DAG_INVOKE_BLOCK, // sqType
    block_hash,  // sqCode
    serialized_payload,
    "Invoke Block(" + cutils::hash16c(block_hash) + ")");

  return status;
}

*/
pub fn getLastReceivedBlockTimestamp() -> JSonObject
{
    let res: String = get_value("LAST_RECEIVED_BLOCK_TIMESTAMP");
    if res == "" {
        return json!({
        "last_block_type": "Genesis",
       "last_block_hash": "-" ,
       "last_block_receive_date": machine().get_launch_date()});
    }
    return cutils::parseToJsonObj(&res);
}

/*

String DAGMessageHandler::getMaybeAskForLatestBlocksFlag()
{
  return KVHandler::getValue("maybe_ask_for_latest_blocks");
}

bool DAGMessageHandler::invokeLeaves()
{
  CLog::log("Invoking for DAG leaves", "app", "trace");
  JSonObject payload {
    {"type", constants::MESSAGE_TYPES::DAG_INVOKE_LEAVES},
    {"mVer", "0.0.0"}};
  String serialized_payload = cutils::serializeJson(payload);

  bool status = SendingQHandler::pushIntoSendingQ(
    constants::MESSAGE_TYPES::DAG_INVOKE_LEAVES, // sqType
    cutils::getNowSSS(),  // sqCode
    serialized_payload,
    "Invoking for DAG leaves");

  return status;
}

void DAGMessageHandler::launchInvokeLeaves()
{
  String shouldI = getMaybeAskForLatestBlocksFlag();
  if (shouldI == constants::YES)
  {
    // TODO: needs control for latest invoke to not spaming network
    invokeLeaves();
    setMaybeAskForLatestBlocksFlag(constants::NO);
  }
}
*/
pub fn setMaybeAskForLatestBlocksFlag(value: &String)
{
    if value == constants::YES {
        // control last_received_leaves_info_timestamp flag
        // if we currently asked for leave information, so do not flood the network with multiple asking
        let last_leave_invoke_response_str: String = get_value("last_received_leaves_info_timestamp");
        if last_leave_invoke_response_str == "" {
            set_value("maybe_ask_for_latest_blocks", value, false);
            return;
        }

        let last_leave_invoke_response: JSonObject = cutils::parseToJsonObj(&last_leave_invoke_response_str);
        // TODO: tune the gap time
        if cutils::time_diff(last_leave_invoke_response["receiveDate"].to_string(), cutils::get_now()).as_seconds < machine().getInvokeLeavesGap() {
            return;
        }

        // control LAST_RECEIVED_BLOCK_TIMESTAMP flag
        // if we are receiving continiuosly new blocks, it doesn't sence to ask for leave information.
        // this case happends in runing a new machin in which the machine has to download entire DAG.
        let last_block: JSonObject = getLastReceivedBlockTimestamp();
        // TODO: tune the gap time
        if cutils::time_diff(last_block["last_block_receive_date"].to_string(), cutils::get_now()).as_seconds < machine().getInvokeLeavesGap() {
            return;
        }

        let machine_request_status: QVDRecordsT = search_in_kv(
            &vec![&simple_eq_clause("kv_key", "maybe_ask_for_latest_blocks")],
            &vec!["kv_last_modified"],
            &vec![],
            0);
        if machine_request_status.len() > 0{
            let invoke_age:TimeBySecT = cutils::time_diff(machine_request_status[0]["kv_last_modified"].to_string(), cutils::get_now()).as_seconds;
            dlog(
                &format!("control if (invoke_age: {} < (invokeGap: {}) ", invoke_age, machine().getInvokeLeavesGap()),
                constants::Modules::App,
                constants::SecLevel::Info);
            if invoke_age < machine().getInvokeLeavesGap() {
                return;
            }
        }
        /*
        // TODO: tune the gap time
        launchInvokeLeaves();
        */
    }

    set_value("maybe_ask_for_latest_blocks", value, false);
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
    block->safeStringifyBlock(false),
    "Replay to invoke for block(" + short_hash + ") type(" + block.m_block_type  + ")",
    {sender});

  CLog::log("invoke block push_res(" + cutils::dumpIt(push_res) + ") block(" + short_hash + ") type(" + block.m_block_type  + ")", "app", "trace");

  return {true, true};
}

 */