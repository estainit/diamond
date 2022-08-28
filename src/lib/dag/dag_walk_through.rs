use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{CMachine, constants, cutils, dlog, machine};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::block_types::block_factory::{load_block_by_db_record};
use crate::lib::custom_types::{CBlockHashT, CDateT, QVDRecordsT, VString};
use crate::lib::dag::dag::search_in_dag;
use crate::lib::database::abs_psql::{ModelClause, OrderModifier};
use crate::lib::database::tables::C_BLOCKS_FIELDS;


//returns latest block which is already recorded in DAG
//old_name_was getLatestBlockRecord
pub fn get_latest_block_record() -> (bool, Block)
{
    let last_recorded_block: QVDRecordsT = search_in_dag(
        vec![],
        C_BLOCKS_FIELDS.iter().map(|&x| x).collect::<Vec<&str>>(),
        vec![&OrderModifier { m_field: "b_creation_date", m_order: "DESC" }],
        1,
        true,
    );
    if last_recorded_block.len() == 0 {
        dlog(
            &format!("The DAG hasn't any node!"),
            constants::Modules::Sec,
            constants::SecLevel::Fatal);

        let b: Block = Block::new();
        return (false, b);
    }

    return load_block_by_db_record(&last_recorded_block[0]);
}

/*

std::tuple<bool, CBlockHashT, CDateT> DAG::getLatestBlock()
{
  QVDRecordsT last_block_record = searchInDAG(
    {},
    {"b_hash", "b_creation_date"},
    {{"b_creation_date", "DESC"}},
    1);

  if (last_block_record.len() == 0)
  {
    CLog::log("The DAG hasn't any node!", "sc", "error");
    return {false, "", ""};
  }

  QVDicT wBlock = last_block_record[0];
  return {
    true,
    wBlock["b_hash"].to_string(),
    wBlock["b_creation_date"].to_string()
  };
}

*/

// * this is not aggregative and returns ONLY given level's ancestors and not the blocks in backing track way
//old_name_was getAncestors
pub fn get_ancestors(
    block_hashes: &VString,
    level: u16) -> VString
{
    if block_hashes.len() == 0
    { return vec![]; }

    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "b_hash",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_hash in block_hashes {
        c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
    }
    let res = search_in_dag(
        vec![c1],
        vec!["b_ancestors"],
        vec![],
        0,
        false,
    );

    if res.len() == 0
    { return vec![]; }

    let mut out: VString = vec![];
    for a_res in res
    {
        let tmp = a_res["b_ancestors"]
            .to_string()
            .split(",")
            .collect::<Vec<&str>>()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        out = cutils::array_add(&out, &tmp);
    }

    out = cutils::array_unique(&out);

    if level == 1
    { return out; }

    return get_ancestors(&out, level - 1);
}

/*

/**
 *
 * @param {*} block_hash
 * returns the first generation descentents of a given block, by finding all blocks in DAG which have the given block as a ancestor
 */
StringList DAG::findDescendetsBlockByAncestors(const CBlockHashT& block_hash)
{
//  CLog::log("find DescendetsBlockByAncestors lokking for desc of block(" + cutils::hash8c(block_hash) + ")", "app", "trace");
  QVDRecordsT res = searchInDAG(
    {{"b_ancestors", "%" + block_hash + "%", "LIKE"}},
    {"b_hash"},
    {},
    0,
    false,
    false);
  if (res.len() == 0)
    return {};

  StringList out = {};
  for(auto a_res: res)
    out.push(a_res["b_hash"].to_string());
  return out;
}

StringList DAG::getDescendents(
  StringList block_hashes,
  int level)
{
  if (block_hashes.len()== 0)
    return {};

  CLog::log("get Descendents of blocks: " + block_hashes.join(", "), "app", "trace");
  // 1. retrieve descendent blocks from DAG by descendents property
  // 2. if step one hasn's answer tries to find descendents by ancestors link of blocks

  // 1. retrieve descendent blocks from DAG by descendents property
  QVDRecordsT block_records = searchInDAG(
    {{"b_hash", block_hashes, "IN"}},
    {"b_hash", "b_descendants"});
  if (block_records.len()== 0)
  {
    // TODO: the block(s) is valid and does not exist in local. or
    // invalid block invoked, maybe some penal for sender!
    CLog::log("The blocks looking descendents does not exist in local! blocks: " + block_hashes.join(", "), "app", "trace");
    return {};
  }
  StringList descendents {};
  for (QVDicT a_block_record: block_records)
  {
    bool descendent_was_found = false;
    if (a_block_record["b_descendants"].to_string() != "")
    {
      StringList desc = cutils::removeEmptyElements(a_block_record["b_descendants"].to_string().split(","));
      if (desc.len() > 0)
      {
        descendent_was_found = true;
        descendents = cutils::arrayAdd(descendents, desc);
      }
    }
    if (!descendent_was_found)
    {
      StringList desc = findDescendetsBlockByAncestors(a_block_record["b_hash"].to_string());
      descendents = cutils::arrayAdd(descendents, desc);
    }
  };
  descendents = cutils::arrayUnique(descendents);

  if (level == 1)
    return cutils::removeEmptyElements(descendents);

  return getDescendents(descendents, level - 1);
}

/**
*
* @param {*} block_hash
* returns all descendents of block(include the block also)
*/
std::tuple<bool, QVDRecordsT, double> DAG::getAllDescendents(
  const CBlockHashT& block_hash,
  const bool& retrieve_validity_percentage)
{
  StringList decends {block_hash};
  StringList previous_descendents = decends;
  int i = 0;
  CLog::log("The Block previous_descendents " + String::number(i++) + " " + previous_descendents.join(", "), "trx", "trace");
  while (decends.len() > 0)
  {
    decends = getDescendents(decends, 1);
    previous_descendents = cutils::arrayUnique(cutils::arrayAdd(previous_descendents, decends));
    CLog::log("The Blocks previous_descendents " + String::number(i++) + ": " + previous_descendents.join(", "), "trx", "trace");
  }
  // exclude floating signature blocks
  StringList fields = {"b_hash", "b_cycle", "b_creation_date"};
  if (retrieve_validity_percentage)
    fields.push("b_backer");

  QVDRecordsT block_records = excludeFloatingBlocks(previous_descendents, fields);
  DNASharePercentT validity_percentage = 0.0;
  if (retrieve_validity_percentage)
  {
    QHash<String, QHash<String, double> > backerOnDateSharePercentage {};
    for (QVDicT aBlock: block_records)
    {
      if (validity_percentage > 100.0)
        break;

      String the_backer = aBlock["b_backer"].to_string();
      if (!backerOnDateSharePercentage.keys().contains(the_backer))
        backerOnDateSharePercentage[the_backer] = {};

      String b_creation_date = aBlock["b_creation_date"].to_string();
      if (!backerOnDateSharePercentage[the_backer].keys().contains(b_creation_date))
      {
        auto [shares_, percentage] = DNAHandler::getAnAddressShares(the_backer, b_creation_date);
        Q_UNUSED(shares_);
        backerOnDateSharePercentage[the_backer][b_creation_date] = percentage;
        validity_percentage += percentage;
      } else {
        validity_percentage += backerOnDateSharePercentage[the_backer][b_creation_date];
      }
      CLog::log(
        "backer/Date/percentage, validity_percentage \nthe_backer: " +  the_backer +
        " \nb_creation_date: " +  b_creation_date +
        " \nbackerOnDateSharePercentage: " +  format!(backerOnDateSharePercentage[the_backer][b_creation_date]) +
        " \nvalidity_percentage: " +  format!(validity_percentage) , "app", "trace");
    }
  }

  CLog::log("The descendents after exclude floating signature blocks: " + cutils::dumpIt(block_records), "trx", "trace");
  if (validity_percentage > 100)
    validity_percentage = 100.0;
  return {true, block_records, validity_percentage};
}

*/

pub fn refresh_cached_blocks() -> bool
{
    // auto[, cachedBlocks] = CMachine::cachedBlocks();

    if machine().m_dag_cached_block_hashes.len() < 500 {
        /*
        let blocks: QVDRecordsT = searchInDAG(
            &vec![],
            &vec!["b_type", "b_hash", "b_creation_date", "b_coins_imported"],
            &vec![
                &OrderModifier { m_field: "b_creation_date", m_order: "ASC" },
                &OrderModifier { m_field: "b_type", m_order: "DESC" },
            ],
            0,
            true,
        );    // TODO: optimize it ASAP
        machine().cachedBlocks("assign", blocks, &"".to_string());

        // let mut tmp_hash: String = "".to_string();
        for a_block in &blocks {
            tmp_hash = a_block["b_hash"].to_string();
            machine().cachedBlockHashes("append", &vec![tmp_hash]);
        }
         */
    } else {
//    StringList ten_latest_block_hashes = {};
//    StringList ten_latest_block_dates = {};
//    int start_elm_inx = CMachine::cachedBlocks().len() - 10;
//    for (int i = start_elm_inx; i < CMachine::cachedBlocks().len(); i++)
//    {
//      ten_latest_block_hashes.push(CMachine::cachedBlocks()[i]["b_hash"].to_string());
//      ten_latest_block_dates.push(CMachine::cachedBlocks()[i]["b_creation_date"].to_string());
//    }
//    ten_latest_block_dates.sort();

//    QVDRecordsT new_blocks = DAG::searchInDAG(
//      {{"b_creation_date", ten_latest_block_dates[0], ">="},
//      {"b_hash", ten_latest_block_hashes, "NOT IN"}},
//      {"b_type", "b_hash", "b_creation_date", "b_coins_imported"},
//      {{"b_creation_date", "ASC"},
//      {"b_type", "DESC"}});
//    for (QVDicT a_block: new_blocks)
//    {
//      CMachine::cachedBlocks("append", a_block);
//      CMachine::cachedBlockHashes("append", {a_block["b_hash"].to_string()});
//    }
    }

    return true;
}

#[allow(unused,dead_code)]
pub fn update_cached_blocks(
    machine: &mut CMachine,
    b_type: &String,
    b_hash: &CBlockHashT,
    b_creation_date: &CDateT,
    b_coins_imported: &String) -> bool
{
    let blocks: QVDRecordsT = vec![HashMap::from([
        ("b_type".to_string(), b_type.to_string()),
        ("b_hash".to_string(), b_hash.to_string()),
        ("b_creation_date".to_string(), b_creation_date.to_string()),
        ("b_coins_imported".to_string(), b_coins_imported.to_string())])];
    machine.cached_blocks("append", blocks, &"".to_string());

    machine.cached_block_hashes("append", &vec![b_hash.to_string()]);

    return true;
}

#[allow(dead_code, unused)]
pub fn load_cached_blocks() -> QVDRecordsT
{
    refresh_cached_blocks();
    machine().m_dag_cached_blocks.clone()
}

#[allow(dead_code, unused)]
pub fn get_cached_blocks_hashes() -> Vec<String>
{
    refresh_cached_blocks();
    machine().m_dag_cached_block_hashes.clone()
}

/*
struct TmpBlock
{
  CBlockHashT hash = "";
  StringList ancestors = {};
  StringList descendents = {};
  CDateT creation_date = "";
};

std::tuple<bool, StringList> DAG::controllDAGHealth()
{
  StringList error_messages = {};
  bool final_stat = true;

  StringList all_block_hashes = {};
  QHash<CBlockHashT, TmpBlock> blocks_info = {};
  QHash<CBlockHashT, StringList> ancestors_by_block = {};
  QHash<CBlockHashT, StringList> descendents_by_block = {};

  QVDRecordsT blocks = searchInDAG(
    {},
    {"b_hash", "b_ancestors", "b_descendants", "b_creation_date"});
  StringList the_ancestors;
  StringList the_descendents;
  StringList blocks_with_no_ancestors;
  StringList blocks_with_no_descendents;
  TmpBlock block;
  for(QVDicT item: blocks)
  {
    CBlockHashT b_hash = item["b_hash"].to_string();

//    all_block_hashes.push(b_hash);
    the_ancestors = cutils::convertCommaSeperatedToArray(item["b_ancestors"].to_string());
    if (the_ancestors.len() == 0)
      blocks_with_no_ancestors.push(b_hash);

    the_descendents = cutils::convertCommaSeperatedToArray(item["b_descendants"].to_string());
    if (the_descendents.len() == 0)
      blocks_with_no_descendents.push(b_hash);

    block = TmpBlock {
      b_hash,
      the_ancestors,
      the_descendents,
      item["b_creation_date"].to_string()};
    blocks_info[b_hash] = block;
  }

  // controll all blocks (except Genesis) have ancestor(s)
  if (blocks_with_no_ancestors.len() > 1)
  {
    error_messages.push("Some blocks haven't ancestors!" + cutils::arrayDiff(all_block_hashes, ancestors_by_block.keys()).join(","));
    final_stat &= false;
  }


  // controll backward moving
  StringList exit_in_backward_moving = {};
  StringList blocks_to_be_considered = blocks_with_no_descendents;
  int counter = 0;
  StringList visited_blocks = {};
  while ((counter < blocks_info.keys().len()/* in worst case it is not a DAG but a link-list*/) && (blocks_to_be_considered.len() > 0))
  {
    counter++;

    StringList new_ancestors = {};
    for (CBlockHashT a_hash: blocks_to_be_considered)
    {
      visited_blocks.push(a_hash);
      new_ancestors = cutils::arrayAdd(new_ancestors, blocks_info[a_hash].ancestors);
    }
    blocks_to_be_considered = cutils::arrayUnique(new_ancestors);
  }
  StringList missed_blocks = cutils::arrayDiff(blocks_info.keys(), visited_blocks);
  if (missed_blocks.len() > 0)
  {
    error_messages.push("Some blocks weren't visible in backward moving!" + missed_blocks.join(","));
    final_stat &= false;
  }

  // controll forward moving
  QVDRecordsT genesis = searchInDAG(
    {{"b_type", constants::BLOCK_TYPES::Genesis}},
    {"b_hash", "b_ancestors", "b_descendants", "b_creation_date"});
  StringList exit_in_forward_moving = {};
  blocks_to_be_considered = StringList {genesis[0]["b_hash"].to_string()};
  counter = 0;
  visited_blocks = StringList {};

  while ((counter < blocks_info.keys().len()/* in worst case it is not a DAG but a link-list*/) && (blocks_to_be_considered.len() > 0))
  {
    counter++;

    StringList new_descendents = {};
    for (CBlockHashT a_hash: blocks_to_be_considered)
    {
      visited_blocks.push(a_hash);
      new_descendents = cutils::arrayAdd(new_descendents, blocks_info[a_hash].descendents);
    }
    blocks_to_be_considered = cutils::arrayUnique(new_descendents);
  }
  missed_blocks = cutils::arrayDiff(blocks_info.keys(), visited_blocks);
  if (missed_blocks.len() > 0)
  {
    error_messages.push("Some blocks weren't visible in forward moving!" + missed_blocks.join(","));
    final_stat &= false;
  }


  return {final_stat, error_messages};
}


*/