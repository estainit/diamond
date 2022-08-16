/*

#include "parsing_q_handler.h"

/**
 * @brief ParsingQHandler::ancestorsConroll
 * @param pq_type
 * @param block
 * @return std::tuple<bool status, bool shouldPurgeMessage>
 */
std::tuple<bool, bool> ParsingQHandler::ancestorsConroll(const String& pq_type, const Block* block)
{

  if (block->m_ancestors.size() == 0)
  {
    CLog::log("The " + pq_type +" Block(" + CUtils::hash16c(block->getBlockHash()) + ") MUST have ancestor!", "sec", "error");
    return {false, true}; //{status, shouldPurgeMessage}
  }

  if (!BlockUtils::ifAncestorsAreValid(block->m_ancestors))
  {
    CLog::log("invalid ancestosr for block(" + CUtils::hash16c(block->getBlockHash())+ ")", "sec", "fatal");
    return { false, true };
  }

  QVDRecordsT existed_record_blocks = DAG::searchInDAG(
    {{"b_hash", block->m_ancestors, "IN"}},
    {"b_hash", "b_creation_date", "b_type", "b_utxo_imported"});
  QStringList existed_hashes;
  for (QVDicT a_block_record: existed_record_blocks)
    existed_hashes.append(a_block_record.value("b_hash").toString());

  QStringList missed_blocks = CUtils::arrayDiff(block->m_ancestors, existed_hashes);
  if (missed_blocks.size() > 0)
  {
    CLog::log("in order to parsing block(" + CUtils::hash16c(block->getBlockHash()) + ") machine needs these missed blocks(" + CUtils::dumpIt(missed_blocks) + ") ");
    appendPrerequisites(block->getBlockHash(), missed_blocks, pq_type);

    // check if the block already is in parsing queue? if not add it to missed blocks to invoke
    QStringList missed_hashes_in_parsing_queue = {};
    for (String hash: missed_blocks)
    {
      QVDRecordsT exists = searchParsingQ(
        {{"pq_code", hash}},
        {"pq_code"});

      if (exists.size() == 0)
        missed_hashes_in_parsing_queue.push_back(hash);
    }
    if (missed_hashes_in_parsing_queue.size() > 0)
    {
      CLog::log("Really missed Blocks, so push to invoking: " + CUtils::dumpIt(missed_hashes_in_parsing_queue));
      MissedBlocksHandler::addMissedBlocksToInvoke(missed_hashes_in_parsing_queue);
    }

    CLog::log(
      "--- Break parsing block because of missed prerequisites block(" +
      CUtils::hash6c(block->getBlockHash()) + ") > " + CUtils::dumpIt(missed_blocks),
      "app", "trace");

    return {false, false};  // must not purge the block until receiving prerquisities blocks
  }

  bool allAncestorsAreImported = true;
  QStringList notImportedAncs = {};
  String oldestAncestorCreationDate = CUtils::getNow();
  for (QVDicT bk: existed_record_blocks)
  {
    // controll ancestors creation date
    if (bk.value("creation_date").toString() > block->m_block_creation_date)
    {
      CLog::log(
        "Block(" + CUtils::hash6c(block->getBlockHash()) + ") " + pq_type +
        " creationDdate(" + block->m_block_creation_date + ") is before it's ancestors(" +
         CUtils::hash6c(bk.value("bHash").toString()) + ") creation Date(" + bk.value("creation_date").toString() + ")",
         "app", "error");

      return {false, true};
    }

    // control import new coins
    if (QStringList {
          CConsts::BLOCK_TYPES::Normal,
          CConsts::BLOCK_TYPES::Coinbase,
          CConsts::BLOCK_TYPES::RpBlock,
          CConsts::BLOCK_TYPES::RlBlock
        }.contains(bk.value("bType").toString())&&
        (bk.value("bUtxoImported").toString() != CConsts::YES))
    {
      allAncestorsAreImported = false;
      notImportedAncs.push_back(bk.value("bHash").toString());
      if (oldestAncestorCreationDate > bk.value("bCreationDate").toString())
        oldestAncestorCreationDate = bk.value("bCreationDate").toString();
    }
  }

  // if is in sync mode, control if ancestors's coins(if exist) are imported
  if (CMachine::isInSyncProcess()&&
    QStringList{CConsts::BLOCK_TYPES::Normal}.contains(block->m_block_type)&&    // in order to let adding FVote blocks to DAG, before importing uplinked Normal block
    !allAncestorsAreImported
    )
  {
    if (CUtils::timeDiff(block->m_block_creation_date).asMinutes < CMachine::getCycleByMinutes() / 6)
    {
      // if block is enoough new maybe machine is not in sync mode more
      CMachine::isInSyncProcess(true);
    }
    // run this controll if the block creation date is not in current sycle
    // infact by passing lastSyncStatus when machine reached to almost leaves in real time
    CLog::log(
      "--- Break parsing block, because of not imported coins of ancestors block(" +
      CUtils::hash6c(block->getBlockHash()) + ") > Ancestors: " + CUtils::dumpIt(notImportedAncs),
      "app", "trace");

//    // manually calling import threads to import ancestors coins (if they are eligible)
//    NormalUTXOHandler::doImportUTXOs(block->m_block_creation_date);
//    CoinbaseUTXOHandler::importCoinbasedUTXOs(oldestAncestorCreationDate);

    return {true, false};

  }

  return {true, true};
}


// appends given Prerequisites for given block
bool ParsingQHandler::appendPrerequisites(
  const String& block_hash,
  const QStringList& prerequisites,
  const String& pq_type)
{
  if (prerequisites.size() == 0)
    return true;

  ClausesT clauses {{"pq_code", block_hash}};
  if (pq_type != "")
    clauses.push_back({"pq_type", pq_type});

  QVDRecordsT res = searchParsingQ(
    clauses,
    {"pq_type", "pq_code", "pq_prerequisites"});

  if (res.size() == 0)
  {
    CLog::log("Wrong requeste to append requisities to a block(" + pq_type + CUtils::hash6c(block_hash) + ") which does not exiss in parsing q!", "sec", "error");
    return false;
  }

  QStringList current_prereq = CUtils::convertJSonArrayToQStringList(CUtils::parseToJsonArr(res[0].value("pq_prerequisites").toString()));
  CLog::log(
    "block(" + CUtils::hash6c(block_hash) + ") adding new prerequisities(" +
    CUtils::dumpIt(prerequisites) + ") to existed prerequisities(" +
    CUtils::dumpIt(current_prereq) + ")", "app", "trace");

  current_prereq = CUtils::arrayAdd(current_prereq, prerequisites);
  current_prereq.sort();
  CLog::log("block(" + CUtils::hash6c(block_hash) + ") final1 prerequisities(" + CUtils::dumpIt(current_prereq) + ")", "app", "trace");
  CLog::log("block(" + CUtils::hash6c(block_hash) + ") final2 prerequisities: " + CUtils::dumpIt(current_prereq), "app", "trace");
  return DbModel::update(
    stbl_parsing_q,
    {
      {"pq_prerequisites", "," + current_prereq.join(",")},
      {"pq_last_modified", CUtils::getNow()}
    },
    {{"pq_code", block_hash}}
  );
}

*/

use crate::lib::custom_types::{ClausesT, LimitT, OrderT, QVDRecordsT};
use crate::lib::database::abs_psql::{pre_query_generator, q_customQuery, q_select, QueryElements};
use crate::lib::database::tables::STBL_PARSING_Q;

pub fn searchParsingQ(
    clauses: &ClausesT,
    fields: &Vec<&str>,
    order: OrderT,
    limit: LimitT) -> QVDRecordsT
{
    let (_status, records) = q_select(
        STBL_PARSING_Q,
        fields,
        clauses,
        order,
        limit,
        true
    );
    // let fields_str: String = fields_array.join(", ");
    // let qElms: QueryElements = pre_query_generator(0, clauses, order, limit);
    // let (_status, records) = q_customQuery(
    //     &("SELECT ".to_owned() + &fields_str + " FROM " + STBL_PARSING_Q + &qElms.m_clauses + &qElms.m_order + &qElms.m_limit),
    //     &qElms.m_params,
    //     true);
    return records;
}
/*

/**
 *
 * @param {*} block_hash
 * NOTE: the queue's prerequisities can be removen ONLY where the referenced block recorded in DAG.
 * in any other cases we must not remove block's prerequisities even the mentioned block already exist in queue
 */
void ParsingQHandler::removePrerequisites(const String& block_hash)
{
  QueryRes res = DbModel::customQuery(
    "",
    "SELECT pq_type, pq_code, pq_prerequisites FROM " + stbl_parsing_q + " WHERE pq_prerequisites LIKE :pq_prerequisites",
    {"pq_type", "pq_code", "pq_prerequisites"},
    0,
    {{"pq_prerequisites", "%" + block_hash + "%"}},
    true,
    false);

  if (res.records.size() == 0)
    return;


  for(QVDicT aBlock: res.records)
  {
    String prerequisites = aBlock.value("pq_prerequisites").toString().replace(block_hash, "");
    prerequisites = CUtils::normalizeCommaSeperatedStr(prerequisites);
    DbModel::update(
      stbl_parsing_q,
      {{"pq_prerequisites", prerequisites}},
      {{"pq_type", aBlock.value("pq_type")},
      {"pq_code", aBlock.value("pq_code")}},
      true,
      false);
  };
}


*/