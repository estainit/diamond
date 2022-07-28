/*

/**
 *
 * @param {string} hashes an array of block hashes
 */
bool MissedBlocksHandler::addMissedBlocksToInvoke(QStringList hashes)
{
  CLog::log("maybe add Missed Blocks To Invoke hashes: "+ CUtils::dumpIt(hashes), "app" "trace");

  if (hashes.size() == 0)
    return true;

  // control if already exist in DAG
  QVDRecordsT existed_in_DAG = DAG::searchInDAG(
  {{"b_hash", hashes, "IN"}},
  {"b_hash"});

  if (existed_in_DAG.size() > 0)
  {
    QStringList existed_in_DAG_hashes = {};
    for(QVDicT a_block: existed_in_DAG)
      existed_in_DAG_hashes.append(a_block.value("b_hash").toString());

    CLog::log("The " + QString::number(existed_in_DAG_hashes.size()) + " of " + QString::number(hashes.size()) + " missed blocks already exist in DAG", "app", "trace");
    hashes = CUtils::arrayDiff(hashes, existed_in_DAG_hashes);
  }

  // control if already exist in missed block table
  QStringList missedBlocks = getMissedBlocksToInvoke();
  missedBlocks = CUtils::arrayUnique(missedBlocks);
  if (missedBlocks.size() > 0)
  {
    CLog::log("The " + QString::number(missedBlocks.size()) + " of " + QString::number(hashes.size()) + " missed blocks already exist in table missed blocks");
    hashes = CUtils::arrayDiff(hashes, missedBlocks);
  }

  // control if already exist in parsing q
  QVDRecordsT existInParse = ParsingQHandler::searchParsingQ(
    {{"pq_code", hashes, "IN"}},
    {"pq_code"});

  if (existInParse.size() > 0)
  {
    QStringList existed_hashes = {};
    for(QVDicT elm: existInParse)
      existed_hashes.append(elm.value("pq_code").toString());

    CLog::log(
      "The " + QString::number(existInParse.size()) + " blocks of seemly missed blocks " +
      QString::number(hashes.size()) + " already exist in table parsing queue",
      "app", "trace");

    hashes = CUtils::arrayDiff(hashes, existed_hashes);
  }

  CLog::log(
    "going to insert missed blocks in miised queue: " + CUtils::dumpIt(hashes),
    "app", "trace");

  for (QString hash: hashes)
  {
    if (hash == "")
      continue;

    QueryRes dbl = DbModel::select(
      stbl_missed_blocks,
      {"mb_block_hash"},
      {{"mb_block_hash", hash}});

    if (dbl.records.size() > 0)
      continue;

    DbModel::insert(
      stbl_missed_blocks,
      {
        {"mb_block_hash", hash},
        {"mb_insert_date", CUtils::getNow()},
        {"mb_last_invoke_date", CUtils::getNow()},
        {"mb_invoke_attempts", 0},
        {"mb_descendents_count", 0}
      }
    );
  }
  return true;
}



QVDRecordsT MissedBlocksHandler::listMissedBlocks(
  QStringList fields,
  const ClausesT& clauses,
  const OrderT& order,
  const int& limit)
{
  if (fields.size() == 0)
    fields = stbl_missed_blocks_fields;

  QueryRes res = DbModel::select(
    stbl_missed_blocks,
    fields,
    clauses,
    order,
    limit);

  return res.records;
}

QStringList MissedBlocksHandler::getMissedBlocksToInvoke(const uint64_t& limit)
{
  QString complete_query = "SELECT mb_block_hash FROM " + stbl_missed_blocks + " ORDER BY mb_invoke_attempts, mb_descendents_count DESC, mb_last_invoke_date, mb_insert_date";
  if (limit != 0) {
    complete_query += " LIMIT " + QString::number(limit);
  }
  QueryRes missed_blocks_res = DbModel::customQuery("", complete_query, {"mb_block_hash"}, 0);
  QStringList missed_hashes = {};
  for(QVDicT a_row: missed_blocks_res.records)
    missed_hashes.append(a_row.value("mb_block_hash").toString());
  return missed_hashes;
}

bool MissedBlocksHandler::removeFromMissedBlocks(const CBlockHashT& block_hash)
{
  DbModel::dDelete(
    stbl_missed_blocks,
    {{"mb_block_hash", block_hash}});
  return true;
}

bool MissedBlocksHandler::increaseAttempNumber(const CBlockHashT& block_hash)
{
  QueryRes attemps = DbModel::select(
    stbl_missed_blocks,
    {"mb_block_hash", "mb_invoke_attempts"},
    {{"mb_block_hash", block_hash}});

  uint attemps_count;
  if (attemps.records.size() > 0)
  {
    attemps_count = attemps.records[0]["mb_invoke_attempts"].toUInt();
  } else {
    attemps_count = 0;
  }

  DbModel::update(
    stbl_missed_blocks,
    {
      {"mb_invoke_attempts", attemps_count + 1},
      {"mb_last_invoke_date", CUtils::getNow()}
    },
    {{"mb_block_hash", block_hash}});

  return true;
}

*/

//old_name_was refreshMissedBlock()
pub fn refresh_missed_block() -> bool
{
    /*
      //aggregate prerequisities in parsing q table and push to missed table 9if doesn's exist on DAG)
      QVDRecordsT records = ParsingQHandler::searchParsingQ(
        {},
        {"pq_code", "pq_prerequisites"});

      QStringList prerequisites = {};
      QStringList existed_in_queue = {};
      for(QVDicT a_record: records)
      {
        existed_in_queue.append(a_record.value("pq_code").toString());
        prerequisites = CUtils::arrayAdd(prerequisites, a_record.value("pq_prerequisites").toString().split(","));
      }

      prerequisites = CUtils::arrayUnique(prerequisites);
      prerequisites = CUtils::arrayDiff(prerequisites, existed_in_queue);

      // insert into missed
      addMissedBlocksToInvoke(prerequisites);
    */
    return true;
}
