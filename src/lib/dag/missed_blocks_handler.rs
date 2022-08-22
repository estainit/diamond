use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, cutils, dlog};
use crate::lib::custom_types::VString;
use crate::lib::dag::dag::search_in_dag;
use crate::lib::database::abs_psql::{ModelClause, q_custom_query, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::C_MISSED_BLOCKS;
use crate::lib::parsing_q_handler::queue_utils::search_parsing_q;

//old_name_was addMissedBlocksToInvoke
pub fn add_missed_blocks_to_invoke(mut hashes: VString) -> bool
{
    dlog(
        &format!("maybe add Missed Blocks To Invoke hashes: {:?}", hashes),
        constants::Modules::App,
        constants::SecLevel::Info);

    if hashes.len() == 0
    { return true; }

    // control if already exist in DAG
    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "b_hash",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_hash in &hashes {
        c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
    }
    let existed_in_dag = search_in_dag(
        vec![c1],
        vec!["b_hash"],
        vec![],
        0,
        false,
    );

    if existed_in_dag.len() > 0
    {
        let mut existed_in_dag_hashes: VString = vec![];
        for a_block in existed_in_dag
        {
            existed_in_dag_hashes.push(a_block["b_hash"].to_string());
        }

        dlog(
            &format!("The {} of {} missed blocks already exist in DAG", existed_in_dag_hashes.len(), hashes.len()),
            constants::Modules::App,
            constants::SecLevel::Info);
        hashes = cutils::array_diff(&hashes, &existed_in_dag_hashes);
    }

    // control if already exist in missed block table
    let mut missed_blocks = get_missed_blocks_to_invoke(0);
    missed_blocks = cutils::array_unique(&missed_blocks);
    if missed_blocks.len() > 0
    {
        dlog(
            &format!("The {} of {} missed blocks already exist in table missed blocks.", missed_blocks.len(), hashes.len()),
            constants::Modules::App,
            constants::SecLevel::Info);
        hashes = cutils::array_diff(&hashes, &missed_blocks);
    }

    // control if already exist in parsing q
    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "pq_code",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_hash in &hashes {
        c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
    }

    let exist_in_parse = search_parsing_q(
        vec![c1],
        vec!["pq_code"],
        vec![],
        0,
    );

    if exist_in_parse.len() > 0
    {
        let mut existed_hashes: VString = vec![];
        for elm in &exist_in_parse
        {
            existed_hashes.push(elm["pq_code"].to_string());
        }

        dlog(
            &format!("The {} blocks of seemly missed blocks {} already exist in table parsing queue", exist_in_parse.len(), hashes.len()),
            constants::Modules::App,
            constants::SecLevel::Info);

        hashes = cutils::array_diff(&hashes, &existed_hashes);
    }

    dlog(
        &format!("going to insert missed blocks in miised queue: {:?}", hashes),
        constants::Modules::App,
        constants::SecLevel::Info);

    for hash in &hashes
    {
        if hash == ""
        { continue; }

        let (status, records) = q_select(
            C_MISSED_BLOCKS,
            vec!["mb_block_hash"],
            vec![simple_eq_clause("mb_block_hash", hash)],
            vec![],
            0,
            false,
        );

        if records.len() > 0
        { continue; }

        let zero: i64 = 0;
        let insert_date = application().get_now();
        let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("mb_block_hash", &hash as &(dyn ToSql + Sync)),
            ("mb_insert_date", &insert_date as &(dyn ToSql + Sync)),
            ("mb_last_invoke_date", &insert_date as &(dyn ToSql + Sync)),
            ("mb_invoke_attempts", &zero as &(dyn ToSql + Sync)),
            ("mb_descendants_count", &zero as &(dyn ToSql + Sync)),
        ]);
        q_insert(
            C_MISSED_BLOCKS,
            &values,
            false,
        );
    }
    return true;
}

/*

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

//old_name_was getMissedBlocksToInvoke
pub fn get_missed_blocks_to_invoke(limit: u64) -> Vec<String>
{
    let mut complete_query: String = "SELECT mb_block_hash FROM ".to_owned() + C_MISSED_BLOCKS + " ORDER BY mb_invoke_attempts, mb_descendants_count DESC, mb_last_invoke_date, mb_insert_date";
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
      {"mb_last_invoke_date", application().get_now()}
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
