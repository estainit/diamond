use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{CMachine, constants, cutils, dlog, machine};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::block_types::block_factory::{load_block_by_db_record};
use crate::lib::custom_types::{CBlockHashT, CDateT, QVDRecordsT, SharesPercentT, VString};
use crate::lib::dag::dag::{exclude_floating_blocks, search_in_dag};
use crate::lib::database::abs_psql::{ModelClause, OrderModifier};
use crate::lib::database::tables::C_BLOCKS_FIELDS;
use crate::lib::services::dna::dna_handler::get_an_address_shares;


//returns latest block which is already recorded in DAG
//old_name_was getLatestBlockRecord
pub fn get_latest_block_record() -> (bool, Block)
{
    let last_recorded_block: QVDRecordsT = search_in_dag(
        vec![],
        Vec::from(C_BLOCKS_FIELDS),
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


// * returns the first generation descentents of a given block, by finding all blocks in DAG which have the given block as a ancestor
//old_name_was findDescendetsBlockByAncestors
pub fn find_descendants_block_by_ancestors(block_hash: &CBlockHashT) -> VString
{
    dlog(
        &format!(
            "find Descendets Block By Ancestors looking for desc of block({})",
            cutils::hash8c(block_hash)),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    let like_clause: String = format!("%{}%", block_hash);
    let records = search_in_dag(
        vec![
            ModelClause {
                m_field_name: "b_ancestors",
                m_field_single_str_value: &like_clause as &(dyn ToSql + Sync),
                m_clause_operand: "LIKE",
                m_field_multi_values: vec![],
            }],
        vec!["b_hash"],
        vec![],
        0,
        false);
    if records.len() == 0
    { return vec![]; }

    let mut out: VString = vec![];
    for a_res in records
    {
        out.push(a_res["b_hash"].clone());
    }
    return out;
}

//old_name_was getDescendents
pub fn get_descendants(
    block_hashes: &VString,
    level: i32) -> VString
{
    if block_hashes.len() == 0
    { return vec![]; }

    dlog(
        &format!(
            "Get descendants of blocks: {}", block_hashes.join(", ")),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    // 1. retrieve descendant blocks from DAG by descendants property
    // 2. if step one hash's answer tries to find descendants by ancestors link of blocks

    // 1. retrieve descendant blocks from DAG by descendants property
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
    let block_records = search_in_dag(
        vec![c1],
        vec!["b_hash", "b_descendants"],
        vec![],
        0,
        false);
    if block_records.len() == 0
    {
        // TODO: the block(s) is valid and does not exist in local. or
        // invalid block invoked, maybe some penal for sender!
        dlog(
            &format!(
                "The blocks looking descendants does not exist in local! blocks: {}", block_hashes.join(", ")),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);
        return vec![];
    }

    let mut descendents: VString = vec![];
    for a_block_record in &block_records
    {
        let mut descendent_was_found: bool = false;
        if a_block_record["b_descendants"] != ""
        {
            let desc: VString = cutils::remove_empty_elements(
                &a_block_record["b_descendants"]
                    .split(",")
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|&x| x.to_string())
                    .collect::<Vec<String>>());
            if desc.len() > 0
            {
                descendent_was_found = true;
                descendents = cutils::array_add(&descendents, &desc);
            }
        }

        if !descendent_was_found
        {
            let desc = find_descendants_block_by_ancestors(&a_block_record["b_hash"]);
            descendents = cutils::array_add(&descendents, &desc);
        }
    };
    descendents = cutils::array_unique(&descendents);

    if level == 1
    {
        return cutils::remove_empty_elements(&descendents);
    }

    return get_descendants(&descendents, level - 1);
}

// * returns all descendents of block(include the block also)
//old_name_was getAllDescendents
pub fn get_all_descendants(
    block_hash: &CBlockHashT,
    retrieve_validity_percentage: bool) -> (bool, QVDRecordsT, f64)
{
    let mut decends: VString = vec![block_hash.to_string()];
    let mut previous_descendents = decends.clone();
    let mut i = 0i32;
    i += 1;
    dlog(
        &format!(
            "The Block previous-descendants {}. {}",
            i, previous_descendents.join(", ")),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    while decends.len() > 0
    {
        i += 1;
        decends = get_descendants(&decends, 1);
        previous_descendents = cutils::array_unique(
            &cutils::array_add(&previous_descendents, &decends));
        dlog(
            &format!(
                "The Blocks previous descendents {}: {}",
                i, previous_descendents.join(", ")),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
    }
    // exclude floating signature blocks
    let mut fields: Vec<&str> = vec!["b_hash", "b_cycle", "b_creation_date"];
    if retrieve_validity_percentage
    {
        fields.push("b_backer");
    }

    let block_records = exclude_floating_blocks(&previous_descendents, fields);
    let mut validity_percentage: SharesPercentT = 0.0;
    if retrieve_validity_percentage
    {
        let mut backer_on_date_share_percentage: HashMap<String, HashMap<String, f64>> = HashMap::new();
        for a_block in &block_records
        {
            if validity_percentage > 100.0
            { break; }

            let the_backer = &a_block["b_backer"];
            if !backer_on_date_share_percentage.contains_key(the_backer)
            {
                backer_on_date_share_percentage.insert(the_backer.clone(), HashMap::new());
            }

            let b_creation_date = &a_block["b_creation_date"];
            if !backer_on_date_share_percentage[the_backer].contains_key(b_creation_date)
            {
                let (_shares, percentage) =
                    get_an_address_shares(the_backer, b_creation_date);
                let mut tmp = backer_on_date_share_percentage[the_backer].clone();
                tmp.insert(b_creation_date.clone(), percentage);
                backer_on_date_share_percentage.insert(the_backer.clone(), tmp);
                validity_percentage += percentage;
            } else {
                validity_percentage += backer_on_date_share_percentage[the_backer][b_creation_date];
            }
            dlog(
                &format!(
                    "backer/Date/percentage, validity_percentage \n the_backer: {} \n b_creation_date: {} \
                    \n backerOnDateSharePercentage: {} \nvalidity_percentage: {}",
                    the_backer,
                    b_creation_date,
                    backer_on_date_share_percentage[the_backer][b_creation_date],
                    validity_percentage),
                constants::Modules::App,
                constants::SecLevel::TmpDebug);
        }
    }

    dlog(
        &format!("The descendants after exclude floating signature blocks: {:?}", block_records),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    if validity_percentage > 100.0
    { validity_percentage = 100.0; }

    return (true, block_records, validity_percentage);
}

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

#[allow(unused, dead_code)]
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
  HashMap<CBlockHashT, TmpBlock> blocks_info = {};
  HashMap<CBlockHashT, StringList> ancestors_by_block = {};
  HashMap<CBlockHashT, StringList> descendents_by_block = {};

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