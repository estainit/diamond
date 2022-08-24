use crate::lib::custom_types::{ClausesT, LimitT, OrderT, QVDRecordsT};
use crate::lib::database::abs_psql::{q_select};
use crate::lib::database::tables::C_PARSING_Q;

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

  if (block->m_ancestors.len() == 0)
  {
    CLog::log("The " + pq_type +" Block(" + cutils::hash16c(block->getBlockHash()) + ") MUST have ancestor!", "sec", "error");
    return {false, true}; //{status, shouldPurgeMessage}
  }

  if (!BlockUtils::ifAncestorsAreValid(block->m_ancestors))
  {
    CLog::log("invalid ancestosr for block(" + cutils::hash16c(block->getBlockHash())+ ")", "sec", "fatal");
    return { false, true };
  }

  QVDRecordsT existed_record_blocks = DAG::searchInDAG(
    {{"b_hash", block->m_ancestors, "IN"}},
    {"b_hash", "b_creation_date", "b_type", "b_utxo_imported"});
  StringList existed_hashes;
  for (QVDicT a_block_record: existed_record_blocks)
    existed_hashes.append(a_block_record.value("b_hash").to_string());

  StringList missed_blocks = cutils::arrayDiff(block->m_ancestors, existed_hashes);
  if (missed_blocks.len() > 0)
  {
    CLog::log("in order to parsing block(" + cutils::hash16c(block->getBlockHash()) + ") machine needs these missed blocks(" + cutils::dumpIt(missed_blocks) + ") ");
    appendPrerequisites(block->getBlockHash(), missed_blocks, pq_type);

    // check if the block already is in parsing queue? if not add it to missed blocks to invoke
    StringList missed_hashes_in_parsing_queue = {};
    for (String hash: missed_blocks)
    {
      QVDRecordsT exists = searchParsingQ(
        {{"pq_code", hash}},
        {"pq_code"});

      if (exists.len() == 0)
        missed_hashes_in_parsing_queue.push_back(hash);
    }
    if (missed_hashes_in_parsing_queue.len() > 0)
    {
      CLog::log("Really missed Blocks, so push to invoking: " + cutils::dumpIt(missed_hashes_in_parsing_queue));
      addMissedBlocksToInvoke(missed_hashes_in_parsing_queue);
    }

    CLog::log(
      "--- Break parsing block because of missed prerequisites block(" +
      cutils::hash6c(block->getBlockHash()) + ") > " + cutils::dumpIt(missed_blocks),
      "app", "trace");

    return {false, false};  // must not purge the block until receiving prerquisities blocks
  }

  bool allAncestorsAreImported = true;
  StringList notImportedAncs = {};
  String oldestAncestorCreationDate = cutils::getNow();
  for (QVDicT bk: existed_record_blocks)
  {
    // controll ancestors creation date
    if (bk.value("creation_date").to_string() > block->m_block_creation_date)
    {
      CLog::log(
        "Block(" + cutils::hash6c(block->getBlockHash()) + ") " + pq_type +
        " creationDdate(" + block->m_block_creation_date + ") is before it's ancestors(" +
         cutils::hash6c(bk.value("bHash").to_string()) + ") creation Date(" + bk.value("creation_date").to_string() + ")",
         "app", "error");

      return {false, true};
    }

    // control import new coins
    if (StringList {
          constants::BLOCK_TYPES::Normal,
          constants::block_types::COINBASE,
          constants::BLOCK_TYPES::RpBlock,
          constants::BLOCK_TYPES::RlBlock
        }.contains(bk.value("bType").to_string())&&
        (bk.value("bUtxoImported").to_string() != constants::YES))
    {
      allAncestorsAreImported = false;
      notImportedAncs.push_back(bk.value("bHash").to_string());
      if (oldestAncestorCreationDate > bk.value("bCreationDate").to_string())
        oldestAncestorCreationDate = bk.value("bCreationDate").to_string();
    }
  }

  // if is in sync mode, control if ancestors's coins(if exist) are imported
  if (CMachine::isInSyncProcess()&&
    StringList{constants::BLOCK_TYPES::Normal}.contains(block->m_block_type)&&    // in order to let adding FVote blocks to DAG, before importing uplinked Normal block
    !allAncestorsAreImported
    )
  {
    if (cutils::timeDiff(block->m_block_creation_date).asMinutes < CMachine::getCycleByMinutes() / 6)
    {
      // if block is enoough new maybe machine is not in sync mode more
      CMachine::isInSyncProcess(true);
    }
    // run this controll if the block creation date is not in current sycle
    // infact by passing lastSyncStatus when machine reached to almost leaves in real time
    CLog::log(
      "--- Break parsing block, because of not imported coins of ancestors block(" +
      cutils::hash6c(block->getBlockHash()) + ") > Ancestors: " + cutils::dumpIt(notImportedAncs),
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
  const StringList& prerequisites,
  const String& pq_type)
{
  if (prerequisites.len() == 0)
    return true;

  ClausesT clauses {{"pq_code", block_hash}};
  if (pq_type != "")
    clauses.push_back({"pq_type", pq_type});

  QVDRecordsT res = searchParsingQ(
    clauses,
    {"pq_type", "pq_code", "pq_prerequisites"});

  if (res.len() == 0)
  {
    CLog::log("Wrong requeste to append requisities to a block(" + pq_type + cutils::hash6c(block_hash) + ") which does not exiss in parsing q!", "sec", "error");
    return false;
  }

  StringList current_prereq = cutils::convertJSonArrayToStringList(cutils::parseToJsonArr(res[0].value("pq_prerequisites").to_string()));
  CLog::log(
    "block(" + cutils::hash6c(block_hash) + ") adding new prerequisities(" +
    cutils::dumpIt(prerequisites) + ") to existed prerequisities(" +
    cutils::dumpIt(current_prereq) + ")", "app", "trace");

  current_prereq = cutils::arrayAdd(current_prereq, prerequisites);
  current_prereq.sort();
  CLog::log("block(" + cutils::hash6c(block_hash) + ") final1 prerequisities(" + cutils::dumpIt(current_prereq) + ")", "app", "trace");
  CLog::log("block(" + cutils::hash6c(block_hash) + ") final2 prerequisities: " + cutils::dumpIt(current_prereq), "app", "trace");
  return DbModel::update(
    C_PARSING_Q,
    {
      {"pq_prerequisites", "," + current_prereq.join(",")},
      {"pq_last_modified", cutils::getNow()}
    },
    {{"pq_code", block_hash}}
  );
}

*/

//old_name_was searchParsingQ
pub fn search_parsing_q(
    clauses: ClausesT,
    fields: Vec<&str>,
    order: OrderT,
    limit: LimitT) -> QVDRecordsT
{
    let (_status, records) = q_select(
        C_PARSING_Q,
        fields,
        clauses,
        order,
        limit,
        true
    );
    // let fields_str: String = fields_array.join(", ");
    // let qElms: QueryElements = pre_query_generator(0, clauses, order, limit);
    // let (_status, records) = q_customQuery(
    //     &("SELECT ".to_owned() + &fields_str + " FROM " + C_PARSING_Q + &qElms.m_clauses + &qElms.m_order + &qElms.m_limit),
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
    "SELECT pq_type, pq_code, pq_prerequisites FROM " + C_PARSING_Q + " WHERE pq_prerequisites LIKE :pq_prerequisites",
    {"pq_type", "pq_code", "pq_prerequisites"},
    0,
    {{"pq_prerequisites", "%" + block_hash + "%"}},
    true,
    false);

  if (res.records.len() == 0)
    return;


  for(QVDicT aBlock: res.records)
  {
    String prerequisites = aBlock.value("pq_prerequisites").to_string().replace(block_hash, "");
    prerequisites = cutils::normalizeCommaSeperatedStr(prerequisites);
    DbModel::update(
      C_PARSING_Q,
      {{"pq_prerequisites", prerequisites}},
      {{"pq_type", aBlock.value("pq_type")},
      {"pq_code", aBlock.value("pq_code")}},
      true,
      false);
  };
}


*/