use std::collections::HashMap;
use std::thread;
use postgres::types::ToSql;
use crate::lib::constants;
use crate::lib::custom_types::{CAddressT, CBlockHashT, CCoinCodeT, CDateT, ClausesT, CMPAIValueT, LimitT, OrderT, QVDRecordsT, VString};
use crate::lib::dlog::dlog;
use crate::{application, cutils, machine};
use crate::lib::database::abs_psql::{clauses_query_generator, ModelClause, q_custom_query, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::C_TRX_COINS;

//old_name_was loopCoinCleaner
#[allow(unused, dead_code)]
pub fn loop_coin_cleaner(c_date: &CDateT)
{
    let thread_prefix = "coin_cleaner_".to_string();
    let thread_code = format!("{:?}", thread::current().id());

    // dlog(
    //     &format!("Going to launch the coin cleaner for {} seconds intervals", machine().get_block_invoke_gap()),
    //     constants::Modules::App,
    //     constants::SecLevel::Info);

    while application().should_loop_threads() {
        machine().report_thread_status(&thread_prefix, &thread_code, &constants::thread_state::RUNNING.to_string());
        do_coin_clean(&"".to_string());

        machine().report_thread_status(&thread_prefix, &thread_code, &constants::thread_state::SLEEPING.to_string());

        // sleep(Duration::from_secs(machine().get_block_invoke_gap()));
    }

    machine().report_thread_status(&thread_prefix, &thread_code, &constants::thread_state::STOPPED.to_string());
    dlog(
        &format!("Gracefully stopped thread({}) of loop coin cleaner", thread_prefix.clone() + &thread_code),
        constants::Modules::App,
        constants::SecLevel::Info);
}


//old_name_was doCoinClean
pub fn do_coin_clean(_c_date: &CDateT)
{
    // * remove from i_trx_utxo the the entries which are visible by blocks that are not "leave" any more and
    // * they are placed 4 level backward(or older) in history
    // * since it is a optional process to lightening DB loadness, it could be done for 8 level previous too

    let _minimum_date: CDateT = application().now();
    // JSonObject leaves = LeavesHandler::getLeaveBlocks();
    /*
  VString leaves_hashes = leaves.keys();
  for (CBlockHashT a_key: leaves_hashes)
    if (minimum_date > leaves[a_key].toObject()["bCDate"].to_string())
      minimum_date = leaves[a_key].toObject()["bCDate"].to_string();

  VString ancestors = get_ancestors(leaves_hashes, 8);
  if (ancestors.len()> 0)
  {
    // to be sure the visibility creation date is 2 cycles older than block creation date
    CDateT min_creation_date = cutils::getCbUTXOsDateRange(minimum_date).from;
    QVDRecordsT block_records = DAG::searchInDAG(
      {{"b_hash", ancestors, "IN"},
      {"b_creation_date", min_creation_date, "<"}},
      {"b_hash"});

    if (block_records.len()> 0)
    {
      for(QVDicT row: block_records)
      {
        CBlockHashT block_hash = row["b_hash"].to_string();
        CLog::log("removing coins which are visible_by " + block_hash, "app", "trace");
        removeVisibleOutputsByBlocks(VString{block_hash}, false);
      }
    }
  }
*/
}

/*
/**
 * method, takes coins visibility and replace them with new blocks in future of block
 */
bool UTXOHandler::refreshVisibility(CDateT c_date)
{
  if (c_date == "")
    c_date = application().now();

  String full_query = "SELECT DISTINCT ut_visible_by, ut_creation_date FROM " + C_TRX_COINS +
  " WHERE ut_creation_date < :ut_creation_date order by ut_creation_date ";
  QueryRes distinct_blocks = DbModel::customQuery(
    "db_comen_spendable_coins",
    full_query,
    {"ut_visible_by", "ut_creation_date"},
    0,
    {{"ut_creation_date", cutils::minutes_before(cutils::get_cycle_by_minutes() * 4, c_date)}},
    true,
    true);

  for (QVDicT a_visibility_entry: distinct_blocks.records)
  {
    CBlockHashT to_be_deleted_blcok = a_visibility_entry["ut_visible_by"].to_string();
    if (!cutils::isValidHash(to_be_deleted_blcok))
    {
      CLog::log("Invalid block hash as to_be_deleted_blcok code! " + to_be_deleted_blcok, "sec", "fatal");
      return false;
    }
    VString block_hashes = get_descendants(VString{to_be_deleted_blcok});  // first generation of descendents
    block_hashes = cutils::arrayAdd(block_hashes, get_descendants(block_hashes));  // second generation
    block_hashes = cutils::arrayAdd(block_hashes, get_descendants(block_hashes));  // third generation
    block_hashes = cutils::array_unique(block_hashes);
    QVDRecordsT descendent_blocks = exclude_floating_blocks(block_hashes); // exclude floating blocks
    CLog::log("visible_bys after exclude floating signature blocks: " + cutils::dumpIt(descendent_blocks), "trx", "trace");

    // avoid duplicate constraint error
    QueryRes candid_res = DbModel::select(
      C_TRX_COINS,
      {"ut_coin"},
      {{"ut_visible_by", to_be_deleted_blcok}},
      {},
      0,
      false,
      false);
    VString candid_coins = {};
    for(QVDicT a_coin: candid_res.records)
      candid_coins.push(a_coin["ut_coin"].to_string());

    for (QVDicT a_descendent_block: descendent_blocks)
    {

      QueryRes existed_res = DbModel::select(
        C_TRX_COINS,
        {"ut_coin"},
        {{"ut_visible_by", a_descendent_block["b_hash"].to_string()}},
        {},
        0,
        false,
        false);

      VString existed_coins {};
      for(QVDicT elm: existed_res.records)
        existed_coins.push(elm["ut_coin"].to_string());

      VString updateables = cutils::array_diff(candid_coins, existed_coins);
      if (updateables.len() > 0)
      {
        // TODO: improve it
        auto updateable_chunks = cutils::chunkStringList(updateables, 100);
        for (VString a_chunk: updateable_chunks)
        {
          DbModel::update(
            C_TRX_COINS,
            {{"ut_visible_by", a_descendent_block["b_hash"].to_string()}},
            {{"ut_visible_by", to_be_deleted_blcok},
            {"ut_coin", a_chunk, "IN"}},
            true,
            false);
        }
      }
    }

    if (descendent_blocks.len() > 0)
    {
      DbModel::dDelete(
        C_TRX_COINS,
        {{"ut_visible_by", to_be_deleted_blcok}},
        true,
        false);

      removeCoinFromCachedSpendableCoins(to_be_deleted_blcok, "");
    }

  }

  return true;
}


ClausesT UTXOHandler::prepareUTXOQuery(
  const VString& coins,
  const VString& visible_by)
{
  ClausesT clauses = {};

  if (coins.len() > 0)
    clauses.push({"ut_coin", coins, "IN"});

  if (visible_by.len() > 0)
    clauses.push({"ut_visible_by", visible_by, "IN"});

  return clauses;
}

*/

//old_name_was searchInSpendableCoinsCache
pub fn search_in_spendable_coins_cache(coins: &VString) -> QVDRecordsT
{
    let mut out: QVDRecordsT = vec![];
    let (status, cached_spendable_coins) =
        machine().cached_spendable_coins(
        "read",
        &vec![],
        &"".to_string(),
        &"".to_string());
    if !status
    {
        dlog(
            &format!("Couldn't read from cached Spendable Coins!"),
            constants::Modules::App,
            constants::SecLevel::Fatal);
    }

    for a_coin in &cached_spendable_coins
    {
        if coins.contains(&a_coin["ut_coin"])
        {
            out.push(a_coin.clone());
        }
    }
    return out;
}

pub fn extract_coins_owner(candidate_coins: &Vec<CCoinCodeT>) -> HashMap<CCoinCodeT, CAddressT>
{
    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "ut_coin",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_coin in candidate_coins {
        c1.m_field_multi_values.push(a_coin as &(dyn ToSql + Sync));
    }
    let coins_info = search_in_spendable_coins(
        &vec![c1],
        &vec![],
        0);
    dlog(
        &format!("selected coins info: {:?}", coins_info),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    let mut out: HashMap<CCoinCodeT, CAddressT> = HashMap::new();
    for a_coin in &coins_info
    {
        out.insert(a_coin["ut_coin"].clone(), a_coin["ut_o_address"].clone());
    }
    dlog(
        &format!("selected coins owner by coin code: {:?}", out),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);
    out
}

//old_name_was searchInSpendableCoins
pub fn search_in_spendable_coins(
    clauses: &ClausesT,
    _order: &OrderT,
    limit: LimitT) -> QVDRecordsT
{
    let (clauses_, values_) = clauses_query_generator(
        0,
        clauses);
    let mut complete_query = format!(
        "SELECT DISTINCT (ut_coin) ut_coin, ut_ref_creation_date, ut_o_address, ut_o_value FROM {} WHERE {}  ",
        C_TRX_COINS,
        clauses_);
    if limit > 0
    {
        complete_query = format!("{} LIMIT {}", limit, complete_query);
    }

    let (_status, records) = q_custom_query(
        &complete_query,
        &values_,
        false,
    );

    return records;
}

// * @param {*} args function clons entire entries are visible_by given block(ancestors)
// * to new entries which are visible_by new-block
//old_name_was inheritAncestorsVisbility
pub fn inherit_ancestors_visbility(
    ancestor_blocks: &VString,
    creation_date: &CDateT,
    new_block_hash: &CBlockHashT)
{
    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "ut_visible_by",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_hash in ancestor_blocks {
        c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
    }
    let (_status, records) = q_select(
        C_TRX_COINS,
        vec!["ut_coin", "ut_o_address", "ut_o_value", "ut_ref_creation_date"],
        vec![c1],
        vec![],
        0,
        false);

    for a_coin in records
    {
        add_new_coin(
            creation_date,
            &a_coin["ut_coin"].to_string(),
            new_block_hash,
            &a_coin["ut_o_address"].to_string(),
            a_coin["ut_o_value"].parse::<CMPAIValueT>().unwrap(),
            &a_coin["ut_ref_creation_date"].to_string());
    }
}


// old name was addNewUTXO
pub fn add_new_coin(
    creation_date: &CDateT,
    the_coin: &CCoinCodeT,
    visible_by: &CBlockHashT,
    address: &CAddressT,
    coin_value: CMPAIValueT,
    coin_creation_date: &CDateT) -> bool
{
    if !cutils::is_valid_hash(&visible_by)
    {
        dlog(
            &format!("Invalid block hash as visibility code! {}", visible_by),
            constants::Modules::Sec,
            constants::SecLevel::Fatal);
        return false;
    }

    if coin_value > constants::MAX_COINS_AMOUNT
    {
        dlog(
            &format!("Invalid coin value to insert! {}", coin_value),
            constants::Modules::Sec,
            constants::SecLevel::Fatal);
        return false;
    }

    // {
    //   // remove this code block after implementing bloom filter
    //   // TODO: implement bloom filter in order to avoid select and reduce the db load ASAP
    //   if (CMachine::cached_coins_visibility().records.len() == 0)
    //   { assignCacheCoinsVisibility(); }
    //
    //   if (CMachine::cached_coins_visibility().records.len() == 0)
    //   {
    //     // why? hwo it dissapeared? BTW re-assign it
    //     assignCacheCoinsVisibility();
    //   }
    //   if (CMachine::cached_coins_visibility("contains", {the_coin + visible_by}).is_visible)
    //     return true;
    // }

    let (_status, records) = q_select(
        C_TRX_COINS,
        vec!["ut_coin"],
        vec![
            simple_eq_clause("ut_coin", the_coin),
            simple_eq_clause("ut_visible_by", visible_by),
        ],
        vec![],
        0,
        false);

    if records.len() > 0
    {
        return true;
    }

    let coin_value_i64 = coin_value as i64;
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("ut_creation_date", &creation_date as &(dyn ToSql + Sync)),
        ("ut_coin", &the_coin as &(dyn ToSql + Sync)),
        ("ut_visible_by", &visible_by as &(dyn ToSql + Sync)),
        ("ut_o_address", &address as &(dyn ToSql + Sync)),
        ("ut_o_value", &coin_value_i64 as &(dyn ToSql + Sync)),
        ("ut_ref_creation_date", &coin_creation_date as &(dyn ToSql + Sync)),
    ]);

    let res = q_insert(
        C_TRX_COINS,
        &values,
        false);

    machine().cached_coins_visibility("append", &vec![format!("{}{}", the_coin, visible_by)]);
    // machine().cached_spendable_coins("append", &vec![values], visible_by, the_coin);
    return res;
}

/*

/**
 *
 * @param {*} hashes
 * Only used for mitigate table load
 */
bool UTXOHandler::removeVisibleOutputsByBlocks(const VString& block_hashes, const bool do_control)
{
  VString unremoveable_blocks = {};
  VString unremoveable_coins = {};

  for (CBlockHashT a_block: block_hashes)
  {
    if (!do_control)
    {
      DbModel::dDelete(
        C_TRX_COINS,
        {{"ut_visible_by", a_block}},
        false,
        false);
      continue;
    }


    QueryRes removing_candidates = DbModel::select(
      C_TRX_COINS,
      {"ut_coin", "ut_visible_by", "ut_creation_date"},
      {{"ut_visible_by", a_block}},
      {},
      0,
      true,
      false);

    // sceptical tests before removing
    // TODO: take care about repayment blocks, since they are created now
    // but block creation date is one cycle before
    for (QVDicT a_coin: removing_candidates.records)
    {
      // control if the utxo already is visible by some newer blocks?
      QueryRes younger_visibility_of_coins = DbModel::select(
        C_TRX_COINS,
        {"ut_coin", "ut_visible_by"},
        {{"ut_coin", a_coin["ut_coin"]},
        {"ut_creation_date", a_coin["ut_creation_date"], ">"}},
        {},
        0,
        true,
        false);

      if (younger_visibility_of_coins.records.len()== 0)
      {
        // security issue
        String msg = "The ut_coin which want to remove can not be seen by newer entries! " + cutils::dumpIt(a_coin);
        CLog::log(msg, "sec", "error");
        unremoveable_blocks.push(a_block);
        unremoveable_coins.push(a_coin["ut_coin"].to_string());
        continue;
      }
      // clog.trx.info(`younger_visibility_of_coins res: ${utils.stringify(younger_visibility_of_coins)}`);

      // if the newer block has the old one in his history?
      bool is_visible_by_ancestors = false;
      for (QVDicT a_visibility: younger_visibility_of_coins.records)
      {
        if (is_visible_by_ancestors)
          continue;

        // retrieve whole ancestors of the utxo
        VString all_ancestors_of_a_younger_block = DAG::returnAncestorsYoungerThan(
          {a_visibility["ut_visible_by"].to_string()},
          a_coin["ut_creation_date"].to_string());

        if (all_ancestors_of_a_younger_block.len() == 0)
          continue;

        if (all_ancestors_of_a_younger_block.contains(a_coin["ut_visible_by"].to_string()))
          is_visible_by_ancestors = true;
      }
      if (!is_visible_by_ancestors)
      {
        // security issue
        String msg = "The ut_coin which want to remove does not exist in history of newer entries! a_coin: " + cutils::dumpIt(a_coin) + " younger Visibility Of RefLoc: " + cutils::dumpIt(younger_visibility_of_coins.records);
        CLog::log(msg, "sec", "error");
        unremoveable_blocks.push(a_block);
        unremoveable_coins.push(a_coin["ut_coin"].to_string());
        continue;
      }

      // finally remove utxo which is visible by his descendents
      if (!unremoveable_blocks.contains(a_block))
      {
        DbModel::dDelete(
          C_TRX_COINS,
          {{"ut_visible_by", a_block},
          {"ut_coin", a_coin["ut_coin"].to_string()}},
          true,
          false);

        removeCoinFromCachedSpendableCoins(a_block, "");
      }

    }
  }

  if ( (unremoveable_blocks.len() > 0) || (unremoveable_coins.len() > 0) )
  {
    unremoveable_blocks = cutils::array_unique(unremoveable_blocks);
    unremoveable_coins = cutils::array_unique(unremoveable_coins);
    CLog::log("There are some unremovable blocks/coins! " + unremoveable_blocks.join(", ") + " " + unremoveable_coins.join(", "), "sec", "error");
    return false;
  }
  return true;
}

// TODO: remove this function after solving database lock problem
void UTXOHandler::removeCoinFromCachedSpendableCoins(
  const CBlockHashT& visible_by,
  const CCoinCodeT& the_coin)
{
  if ((visible_by == "") && (the_coin==""))
    return;

  QVDRecordsT remined_coins = {};
  auto [status, current_cache] = CMachine::cached_spendable_coins("remove", {}, visible_by, the_coin);
}

bool UTXOHandler::removeCoin(const CCoinCodeT& the_coin)
{
//  CLog::log("remove an spent coin(" + cutils::short_coin_code(the_coin) + ")", "trx", "trace");
  DbModel::dDelete(
    C_TRX_COINS,
    {{"ut_coin", the_coin}},
    true,
    false);

  removeCoinFromCachedSpendableCoins("", the_coin);

  return true;
}


bool UTXOHandler::removeUsedCoinsByBlock(const Block* block)
{
  CLog::log("remove spent UXTOs of Block(" + cutils::hash8c(block->getBlockHash()) + ")", "trx", "trace");
  for (Document* doc: block->getDocuments())
    for (TInput* input: doc->get_inputs())
      removeCoin(input.get_coin_code());
  return true;
}

std::tuple<CMPAIValueT, QVDRecordsT, QV2DicT> UTXOHandler::getSpendablesInfo()
{
  QueryRes res = DbModel::select(
    C_TRX_COINS,
    {"ut_coin", "ut_o_value", "ut_ref_creation_date", "ut_visible_by"},
    {},
    {{"ut_ref_creation_date", "ASC"},
    {"ut_o_value", "DESC"}},
    0,
    true,
    false);

  CMPAIValueT sum = 0;
  QVDRecordsT utxos = {};
  QV2DicT utxos_dict = {};
  for (QVDicT a_coin: res.records)
  {
    CCoinCodeT the_coin = a_coin["ut_coin"].to_string();
    if (!utxos_dict.keys().contains(the_coin))
    {
      utxos_dict[the_coin] = QVDicT {
        {"refLocCreationDate", a_coin["ut_ref_creation_date"].to_string()},
        {"outValue", a_coin["ut_o_value"].toDouble()},
        {"visibleBy", VString{}}};

      utxos.push(QVDicT {
        {"refLoc", the_coin},
        {"refLocCreationDate", a_coin["ut_ref_creation_date"].to_string()},
        {"outValue", a_coin["ut_o_value"].toDouble()}});

      sum += a_coin["ut_o_value"].toDouble();
    }
    auto tmp = utxos_dict[the_coin]["visibleBy"].toList();
    tmp.push(a_coin["ut_visible_by"]);
    utxos_dict[the_coin]["visibleBy"] = tmp;
  }
  return {sum, utxos, utxos_dict };
}

*/
//old_name_was extractUTXOsBYAddresses
#[allow(unused, dead_code)]
pub fn extract_coins_by_addresses(addresses: &VString) -> QVDRecordsT
{
    if addresses.len() == 0
    { return vec![]; }

    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "ut_o_address",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for an_address in addresses {
        c1.m_field_multi_values.push(an_address as &(dyn ToSql + Sync));
    }

    let tmp_clauses = vec![c1];
    let (clauses_, values_) = clauses_query_generator(
        0,
        &tmp_clauses);

    let complete_query = format!(
        "SELECT ut_coin, ut_o_address, ut_o_value, min(ut_ref_creation_date) AS ref_creation_date \
        FROM {} WHERE {} GROUP BY ut_coin, ut_o_address, ut_o_value \
        ORDER BY min(ut_ref_creation_date), ut_o_address, ut_o_value",
        C_TRX_COINS, clauses_);

    let (_status, records) = q_custom_query(
        &complete_query,
        &values_,
        false);
    let mut new_coins: QVDRecordsT = vec![];
    for element in records
    {
        let a_coin = HashMap::from([
            ("ut_ref_creation_date".to_string(), element["ref_creation_date"].clone()),
            ("ut_coin".to_string(), element["ut_coin"].clone()),
            ("ut_o_address".to_string(), element["ut_o_address"].clone()),
            ("ut_o_value".to_string(), element["ut_o_value"].clone())
        ]);
        new_coins.push(a_coin);
    };
    return new_coins;
}

/*
QVDRecordsT UTXOHandler::generateCoinsVisibilityReport()
{
  String complete_query = "SELECT DISTINCT ut_visible_by, ut_coin, ut_o_address, ut_o_value FROM " + C_TRX_COINS + " ORDER BY ut_visible_by, ut_coin, ut_o_address, ut_o_value";
  QueryRes utxos = DbModel::customQuery(
    "db_comen_spendable_coins",
    complete_query,
    {"ut_visible_by", "ut_coin", "ut_o_address", "ut_o_value"},
    0,
    {},
    true,
    false);

  QV2DicT visibility = {};
  HashMap<CBlockHashT, Vec<String> > coins_list = {};
  HashMap<CBlockHashT, Vec<String> > owners_list = {};
  HashMap<CCoinCodeT, CMPAIValueT> map_coins_to_value = {};

  for (QVDicT a_row: utxos.records)
  {
    CBlockHashT visible_by = a_row["ut_visible_by"].to_string();
    CCoinCodeT the_coin = a_row["ut_coin"].to_string();
    CAddressT address = a_row["ut_o_address"].to_string();
    CMPAIValueT coin_value = a_row["ut_o_value"].toDouble();

    if (!coins_list.keys().contains(visible_by))
    {
      coins_list[visible_by] = {};
      owners_list[visible_by] = {};
    }

    map_coins_to_value[the_coin] = coin_value;

    coins_list[visible_by].push(the_coin);
    owners_list[visible_by].push(the_coin);
  }

  // prepare block info
  VString blocks_hashes = coins_list.keys();
  QV2DicT blocks_dict = {};
  if (blocks_hashes.len() > 0)
  {
    QVDRecordsT blocks_info = DAG::searchInDAG(
      {{"b_hash", blocks_hashes, "IN"}},
      {"b_hash", "b_type", "b_creation_date"},
      {{"b_creation_date", "ASC"},
      {"b_type", "ASC"},
      {"b_hash", "ASC"}});
    for (QVDicT b: blocks_info)
      blocks_dict[b["b_hash"].to_string()] = QVDicT {
        {"block_type", b["b_type"].to_string()},
        {"block_creation_date", b["b_creation_date"].to_string()}};
  }


  for (CBlockHashT a_block :blocks_hashes)
  {
    CMPAIValueT coins_value = 0;
    for (CCoinCodeT a_coin: coins_list[a_block])
      coins_value += map_coins_to_value[a_coin];

    // unify and sort coins
    std::sort(coins_list[a_block].begin(), coins_list[a_block].end());
    auto last = std::unique(coins_list[a_block].begin(), coins_list[a_block].end());
    coins_list[a_block].erase(last, coins_list[a_block].end());
    String coins_str = "";
    for(String a_coin: coins_list[a_block])
      coins_str += a_coin;
    String coins_hash = ccrypto::keccak256(coins_str);

    // unify and sort owners
    std::sort(owners_list[a_block].begin(), owners_list[a_block].end());
    auto last_o = std::unique(owners_list[a_block].begin(), owners_list[a_block].end());
    owners_list[a_block].erase(last_o, owners_list[a_block].end());
    String owners_str = "";
    for(String an_owner: owners_list[a_block])
      owners_str += an_owner;
    String owners_hash = ccrypto::keccak256(owners_str);


    String the_key = blocks_dict[a_block]["block_creation_date"].to_string() + coins_hash + a_block;
    visibility[the_key] = QVDicT {
      {"the_key", the_key},
      {"visible_by", a_block},
      {"block_type", blocks_dict[a_block]["block_type"]},
      {"block_creation_date", blocks_dict[a_block]["block_creation_date"]},
      {"coins_count", QVariant::fromValue(coins_list[a_block].len())},
      {"coins_hash", coins_hash},
      {"owners_count", QVariant::fromValue(owners_list[a_block].len())},
      {"owners_hash", ccrypto::keccak256(owners_hash)},
      {"coins_value", String::number(coins_value)}};
  }
  QVDRecordsT out = {};
  VString keys = visibility.keys();
  keys.sort();
  for(String key: keys)
    out.push(visibility[key]);

  return out;
}

void UTXOHandler::assignCacheCoinsVisibility()
{
  QueryRes existed = DbModel::select(
    C_TRX_COINS,
    {"ut_coin", "ut_visible_by"});
  String tmp_vis_coin = "";
  for (QVDicT elm: existed.records)
  {
    tmp_vis_coin = elm["ut_coin"].to_string() + elm["ut_visible_by"].to_string();
    CMachine::cached_coins_visibility("append", {tmp_vis_coin});
  }

}

 */