/*

/**
 *
 * @param {string} hashes an array of block hashes
 */
bool MissedBlocksHandler::addMissedBlocksToInvoke(StringList hashes)
{
  CLog::log("maybe add Missed Blocks To Invoke hashes: "+ cutils::dumpIt(hashes), "app" "trace");

  if (hashes.len() == 0)
    return true;

  // control if already exist in DAG
  QVDRecordsT existed_in_DAG = DAG::searchInDAG(
  {{"b_hash", hashes, "IN"}},
  {"b_hash"});

  if (existed_in_DAG.len() > 0)
  {
    StringList existed_in_DAG_hashes = {};
    for(QVDicT a_block: existed_in_DAG)
      existed_in_DAG_hashes.push(a_block["b_hash"].to_string());

    CLog::log("The " + String::number(existed_in_DAG_hashes.len()) + " of " + String::number(hashes.len()) + " missed blocks already exist in DAG", "app", "trace");
    hashes = cutils::arrayDiff(hashes, existed_in_DAG_hashes);
  }

  // control if already exist in missed block table
  missedBlocks = cutils::arrayUnique(missedBlocks);
  if (missedBlocks.len() > 0)
  {
    CLog::log("The " + String::number(missedBlocks.len()) + " of " + String::number(hashes.len()) + " missed blocks already exist in table missed blocks");
    hashes = cutils::arrayDiff(hashes, missedBlocks);
  }

  // control if already exist in parsing q
  QVDRecordsT existInParse = ParsingQHandler::searchParsingQ(
    {{"pq_code", hashes, "IN"}},
    {"pq_code"});

  if (existInParse.len() > 0)
  {
    StringList existed_hashes = {};
    for(QVDicT elm: existInParse)
      existed_hashes.push(elm["pq_code"].to_string());

    CLog::log(
      "The " + String::number(existInParse.len()) + " blocks of seemly missed blocks " +
      String::number(hashes.len()) + " already exist in table parsing queue",
      "app", "trace");

    hashes = cutils::arrayDiff(hashes, existed_hashes);
  }

  CLog::log(
    "going to insert missed blocks in miised queue: " + cutils::dumpIt(hashes),
    "app", "trace");

  for (String hash: hashes)
  {
    if (hash == "")
      continue;

    QueryRes dbl = DbModel::select(
      STBL_MISSED_BLOCKS,
      {"mb_block_hash"},
      {{"mb_block_hash", hash}});

    if (dbl.records.len() > 0)
      continue;

    DbModel::insert(
      STBL_MISSED_BLOCKS,
      {
        {"mb_block_hash", hash},
        {"mb_insert_date", cutils::get_now()},
        {"mb_last_invoke_date", cutils::get_now()},
        {"mb_invoke_attempts", 0},
        {"mb_descendants_count", 0}
      }
    );
  }
  return true;
}



QVDRecordsT MissedBlocksHandler::listMissedBlocks(
  StringList fields,
  const ClausesT& clauses,
  const OrderT& order,
  const int& limit)
{
  if (fields.len() == 0)
    fields = STBL_MISSED_BLOCKS_fields;

  QueryRes res = DbModel::select(
    STBL_MISSED_BLOCKS,
    fields,
    clauses,
    order,
    limit);

  return res.records;
}
*/
use crate::lib::database::abs_psql::q_custom_query;
use crate::lib::database::tables::STBL_MISSED_BLOCKS;

//old_name_was getMissedBlocksToInvoke
pub fn get_missed_blocks_to_invoke(limit: u64) -> Vec<String>
{
    let mut complete_query:String = "SELECT mb_block_hash FROM ".to_owned() + STBL_MISSED_BLOCKS + " ORDER BY mb_invoke_attempts, mb_descendants_count DESC, mb_last_invoke_date, mb_insert_date";
    if limit != 0 {
        complete_query += &*(" LIMIT ".to_owned() + &limit.to_string());
    }
    let (_status, records) = q_custom_query(&complete_query, &vec![], true);
    let mut missed_hashes: Vec<String> = vec![];
    for a_row in records
    {
        missed_hashes.push(a_row["mb_block_hash"].to_string());
    }
    return missed_hashes;
}

/*

bool MissedBlocksHandler::removeFromMissedBlocks(const CBlockHashT& block_hash)
{
  DbModel::dDelete(
    STBL_MISSED_BLOCKS,
    {{"mb_block_hash", block_hash}});
  return true;
}

bool MissedBlocksHandler::increaseAttempNumber(const CBlockHashT& block_hash)
{
  QueryRes attemps = DbModel::select(
    STBL_MISSED_BLOCKS,
    {"mb_block_hash", "mb_invoke_attempts"},
    {{"mb_block_hash", block_hash}});

  uint attemps_count;
  if (attemps.records.len() > 0)
  {
    attemps_count = attemps.records[0]["mb_invoke_attempts"].toUInt();
  } else {
    attemps_count = 0;
  }

  DbModel::update(
    STBL_MISSED_BLOCKS,
    {
      {"mb_invoke_attempts", attemps_count + 1},
      {"mb_last_invoke_date", cutils::get_now()}
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

      StringList prerequisites = {};
      StringList existed_in_queue = {};
      for(QVDicT a_record: records)
      {
        existed_in_queue.push(a_record["pq_code"].to_string());
        prerequisites = cutils::arrayAdd(prerequisites, a_record["pq_prerequisites"].to_string().split(","));
      }

      prerequisites = cutils::arrayUnique(prerequisites);
      prerequisites = cutils::arrayDiff(prerequisites, existed_in_queue);

      // insert into missed
      addMissedBlocksToInvoke(prerequisites);
    */
    return true;
}
