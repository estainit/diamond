/*
#include "stable.h"
#include "lib/sending_q_handler/sending_q_handler.h"
#include "lib/parsing_q_handler/parsing_q_handler.h"
#include "lib/dag/full_dag_handler.h"
#include "dag_message_handler.h"

const QString DAGMessageHandler::stbl_kvalue = "c_kvalue";

DAGMessageHandler::DAGMessageHandler()
{

}

bool DAGMessageHandler::setLastReceivedBlockTimestamp(
  const QString &bType,
  const QString &block_hash,
  const QString &receive_date)
{
  DbModel::upsert(
    stbl_kvalue,
    "kv_key",
    "LAST_RECEIVED_BLOCK_TIMESTAMP",
    {
      {"kv_value", CUtils::serializeJson({
        {"last_block_type", bType},
        {"last_block_hash", block_hash},
        {"last_block_receive_date", receive_date}
      })},
      {"kv_last_modified", CUtils::getNow()}
    });
  return true;
}

bool DAGMessageHandler::invokeDescendents(
  const bool &denay_double_send_check)
{
  // read latest recorded block in DAG
  auto[status, block_hash, block_creation_date] = DAG::getLatestBlock();
  Q_UNUSED(status);

  if (CUtils::timeDiff(block_creation_date).asMinutes > CMachine::getAcceptableBlocksGap())
  {
    // control if block's potentially descendent(s) exist in parsing q
    QVDRecordsT likeHashRes = ParsingQHandler::searchParsingQ(
      {
        {"pq_type", {CConsts::BLOCK_TYPES::Normal, CConsts::BLOCK_TYPES::Coinbase}, "IN"},
        {"pq_code", block_hash}
      },
      {"pq_type", "pq_code", "pq_payload"});

    // invoke network for block probably descendents
    QStringList existed_descendents_in_parsingQ = {};
    if (likeHashRes.size() > 0)
    {
      for (QVDicT wBlock: likeHashRes)
      {
        QJsonObject jBlock = CUtils::parseToJsonObj(wBlock.value("pq_payload").toString());
        // if the existed block in parsing q is descendent of block
        QStringList tmp = {};
        for(QJsonValueRef an_anc: jBlock.value("ancestors").toArray())
          tmp.push_back(an_anc.toString());
        if (tmp.contains(block_hash))
          existed_descendents_in_parsingQ.push_back(jBlock.value("bHash").toString());
      }
    }
    if (existed_descendents_in_parsingQ.size() > 0)
    {
      // controling if the ancestors of descendent exist in local or not
      existed_descendents_in_parsingQ = CUtils::arrayUnique(existed_descendents_in_parsingQ);
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
  const QString &block_hash,
  const QString &block_creation_date,
  const bool &denay_double_send_check)
{
  CLog::log("do Invoke Descendents args block_hash(" + block_hash + ") block_creation_date(" + block_creation_date + ") denay_double_send_check(" + CUtils::dumpIt(denay_double_send_check) + ")", "app", "trace");

  // if the last block which exists in DAG is older than 2 cycle time maybe efficient to call full-history
  if (block_creation_date < CUtils::minutesBefore(2 * CMachine::getCycleByMinutes()))
  {
    QString LastFullDAGDownloadResponse = KVHandler::getValue("LAST_FULL_DAG_DOWNLOAD_RESPONSE");
    if (LastFullDAGDownloadResponse == "")
    {
      KVHandler::upsertKValue("LAST_FULL_DAG_DOWNLOAD_RESPONSE", CUtils::minutesBefore(CMachine::getCycleByMinutes()));

    } else {
      if (CUtils::timeDiff(LastFullDAGDownloadResponse).asMinutes < 5)
      {
        CLog::log("less than 5 minutes ago invoked for full DAG", "app", "trace");
        return true;
      }
    }

    // TODO: improve it to not send full req to all neighbors
    FullDAGHandler::invokeFullDAGDlRequest(block_creation_date);


  } else {

    CLog::log("invoking for descendents of ${utils.hash6c(block_hash)}", "app", "trace");
    QJsonObject payload = {
      {"type", CConsts::MESSAGE_TYPES::DAG_INVOKE_DESCENDENTS},
      {"mVer", "0.0.0"},
      {"bHash", block_hash}
    };
    QString payload_ = CUtils::serializeJson(payload);
    SendingQHandler::pushIntoSendingQ(
      CConsts::MESSAGE_TYPES::DAG_INVOKE_DESCENDENTS,
      block_hash, // sqCode
      payload_,
      "Invoke Descendents(" + CUtils::hash16c(block_hash) + ")",
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
  QStringList block_hashes,
  uint level)
{


  QStringList next_level_block_hashes = {};
  QStringList missed_blocks = {};
  for (uint l = 0; l < level; l++)
  {
    // exists in DAG?
    QVDRecordsT existedInDAG = DAG::searchInDAG(
      {{"b_hash", block_hashes, "IN"}},
      {"b_hash"});
    if (existedInDAG.size() == block_hashes.size())
      continue; // all blocks are already recorded in local graph

    QStringList tmp;
    for(QVDicT a_row: existedInDAG)
      tmp.append(a_row.value("b_hash").toString());
    QStringList array_diff = CUtils::arrayDiff(block_hashes, tmp);

    // control if block exist in parsing_q
    for (auto looking_hash: array_diff)
    {
      QVDRecordsT existsInParsingQ = ParsingQHandler::searchParsingQ(
        {{"pq_code", looking_hash}},
        {"pq_code", "pq_payload"});

      if (existsInParsingQ.size() == 0)
      {
        missed_blocks.push_back(looking_hash);
      } else {
//        let ancestors = existsInParsingQ.map(x => JSON.parse(x.pqPayload).ancestors);
        QList<QStringList> ancestors;
        for(auto x: existsInParsingQ)
        {
          QJsonObject payloadJs = CUtils::parseToJsonObj(x.value("pq_payload").toString());
          QJsonArray ancsJS = payloadJs.value("ancestors").toArray();
          QStringList ancestors;
          for(auto y: ancsJS)
            ancestors.append(y.toString());
        }

        if (ancestors.size() == 0)
        {
          CLog::log("The block(" + CUtils::hash16c(looking_hash) + ") has no valid ancestors! " + CUtils::dumpIt(existsInParsingQ), "sec", "error");
          return false;
        }
        for(auto pckedAncestors: ancestors)
        {
          for(auto ancestor: pckedAncestors)
          {
            next_level_block_hashes.push_back(ancestor);
          }
        }
      }
    }
    block_hashes = CUtils::arrayUnique(next_level_block_hashes);
  }
  missed_blocks = CUtils::arrayUnique(missed_blocks);
  MissedBlocksHandler::addMissedBlocksToInvoke(missed_blocks);
//  loopMissedBlocksInvoker();
  return true;
}

void DAGMessageHandler::loopMissedBlocksInvoker()
{
  QString thread_prefix = "missed_blocks_invoker_";
  QString thread_code = QString::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::RUNNING);
    doMissedBlocksInvoker();

    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getBlockInvokeGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Missed Blocks Invoker");
}
*/
//old_name_was doMissedBlocksInvoker
pub fn do_missed_blocks_invoker()
{
    /*
  QString cycle = CUtils::getCoinbaseCycleStamp();
  CLog::log("ReMiBcInv cycle(" + cycle + ") called recursive MissedBlocks Invoker", "app", "trace");
  QStringList missed = MissedBlocksHandler::getMissedBlocksToInvoke(2);
  // listener.doCallAsync('APSH_control_if_missed_block');

  if (missed.size() > 0)
  {
    CLog::log("ReMiBcInv cycle(" + cycle + ") recursive Missed Blocks Invoker has " + QString::number(missed.size()) + " missed blocks(" + CUtils::dumpIt(missed) + ")", "app", "trace");
    for(QString a_missed: missed)
    {
      //check if not already exist in parsing q
      QVDRecordsT existsInParsingQ = ParsingQHandler::searchParsingQ(
        {{"pq_code", a_missed}},
        {"pq_type", "pq_code"});

      if (existsInParsingQ.size() == 0)
      {
        invokeBlock(a_missed);
        MissedBlocksHandler::increaseAttempNumber(a_missed);
      }
    }
  }
    */
}
/*
bool DAGMessageHandler::invokeBlock(const QString &block_hash)
{
  CLog::log("invoking for block(" + CUtils::hash16c(block_hash) + ")", "app", "trace");
  QJsonObject payload {
    {"type", CConsts::MESSAGE_TYPES::DAG_INVOKE_BLOCK},
    {"mVer", "0.0.0"},
    {"bHash", block_hash}};
  QString serialized_payload = CUtils::serializeJson(payload);
  CLog::log("invoked for keaves (" + block_hash + ")", "app", "trace");

  bool status = SendingQHandler::pushIntoSendingQ(
    CConsts::MESSAGE_TYPES::DAG_INVOKE_BLOCK, // sqType
    block_hash,  // sqCode
    serialized_payload,
    "Invoke Block(" + CUtils::hash16c(block_hash) + ")");

  return status;
}


QJsonObject DAGMessageHandler::getLastReceivedBlockTimestamp()
{
  QString res = KVHandler::getValue("LAST_RECEIVED_BLOCK_TIMESTAMP");
  if (res == "")
    return QJsonObject {
      {"last_block_type", "Genesis"},
      {"last_block_hash", "-"},
      {"last_block_receive_date", CMachine::getLaunchDate()}};
  return CUtils::parseToJsonObj(res);
}

QString DAGMessageHandler::getMaybeAskForLatestBlocksFlag()
{
  return KVHandler::getValue("maybe_ask_for_latest_blocks");
}

bool DAGMessageHandler::invokeLeaves()
{
  CLog::log("Invoking for DAG leaves", "app", "trace");
  QJsonObject payload {
    {"type", CConsts::MESSAGE_TYPES::DAG_INVOKE_LEAVES},
    {"mVer", "0.0.0"}};
  QString serialized_payload = CUtils::serializeJson(payload);

  bool status = SendingQHandler::pushIntoSendingQ(
    CConsts::MESSAGE_TYPES::DAG_INVOKE_LEAVES, // sqType
    CUtils::getNowSSS(),  // sqCode
    serialized_payload,
    "Invoking for DAG leaves");

  return status;
}

void DAGMessageHandler::launchInvokeLeaves()
{
  QString shouldI = getMaybeAskForLatestBlocksFlag();
  if (shouldI == CConsts::YES)
  {
    // TODO: needs control for latest invoke to not spaming network
    invokeLeaves();
    setMaybeAskForLatestBlocksFlag(CConsts::NO);
  }
}

void DAGMessageHandler::setMaybeAskForLatestBlocksFlag(const QString& value)
{
  if (value == CConsts::YES)
  {
    // control last_received_leaves_info_timestamp flag
    // if we currently asked for leave information, so do not flood the network with multiple asking
    QString last_leave_invoke_response_str = KVHandler::getValue("last_received_leaves_info_timestamp");
    if (last_leave_invoke_response_str == "")
    {
      KVHandler::setValue("maybe_ask_for_latest_blocks", value);
      return;
    }

    QJsonObject last_leave_invoke_response = CUtils::parseToJsonObj(last_leave_invoke_response_str);
    // TODO: tune the gap time
    if (CUtils::timeDiff(last_leave_invoke_response.value("receiveDate").toString()).asSeconds < CMachine::getInvokeLeavesGap())
      return;

    // control LAST_RECEIVED_BLOCK_TIMESTAMP flag
    // if we are receiving continiuosly new blocks, it doesn't sence to ask for leave information.
    // this case happends in runing a new machin in which the machine has to download entire DAG.
    QJsonObject last_block = getLastReceivedBlockTimestamp();
    // TODO: tune the gap time
    if (CUtils::timeDiff(last_block.value("last_block_receive_date").toString()).asSeconds < CMachine::getInvokeLeavesGap())
      return;

    QVDRecordsT machine_request_status = KVHandler::serach(
      {{"kv_key", "maybe_ask_for_latest_blocks"}},
      {"kv_last_modified"});
    if (machine_request_status.size() > 0)
    {
      TimeBySecT invoke_age = CUtils::timeDiff(machine_request_status[0].value("kv_last_modified").toString()).asSeconds;
      CLog::log("control if (invoke_age: " + QString::number(invoke_age) + ") < (invokeGap: " + QString::number(CMachine::getInvokeLeavesGap()) + ") ");
      if (invoke_age < CMachine::getInvokeLeavesGap())
          return;

    }
    // TODO: tune the gap time
    launchInvokeLeaves();
  }

  KVHandler::setValue("maybe_ask_for_latest_blocks", value);
}



std::tuple<bool, bool> DAGMessageHandler::handleBlockInvokeReq(
  const QString& sender,
  const QJsonObject& payload,
  const QString& connection_type)
{
  const CBlockHashT& block_hash = payload.value("bHash").toString();

  QString short_hash = CUtils::hash8c(block_hash);
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

  CLog::log("Broadcasting Replay to invoke for block(" + regenerated_json_block.value("bType").toString() + " / " + CUtils::hash8c(block_hash) + ")", "app", "trace");
  Block* block = BlockFactory::create(regenerated_json_block);
  bool push_res = SendingQHandler::pushIntoSendingQ(
    block->m_block_type,
    block_hash,
    block->safeStringifyBlock(false),
    "Replay to invoke for block(" + short_hash + ") type(" + block->m_block_type  + ")",
    {sender});

  CLog::log("invoke block push_res(" + CUtils::dumpIt(push_res) + ") block(" + short_hash + ") type(" + block->m_block_type  + ")", "app", "trace");

  return {true, true};
}

 */