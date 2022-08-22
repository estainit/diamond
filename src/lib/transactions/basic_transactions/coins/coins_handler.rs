use std::thread;
use crate::lib::constants;
use crate::lib::custom_types::CDateT;
use crate::lib::dlog::dlog;
use crate::{application, machine};

/*

pub static stbl_trx_utxos: &str = "c_trx_utxos";
pub static stbl_trx_utxos_fields: Vec<&str> = vec!["ut_id", "ut_creation_date", "ut_coin", "ut_o_address", "ut_o_value", "ut_visible_by", "ut_ref_creation_date"];
 */

//old_name_was loopCoinCleaner
pub fn loop_coin_cleaner(c_date: &CDateT)
{
    let thread_prefix = "coin_cleaner_".to_string();
    let thread_code = format!("{:?}", thread::current().id());

    // dlog(
    //     &format!("Going to launch the coin cleaner for {} seconds intervals", machine().get_block_invoke_gap()),
    //     constants::Modules::App,
    //     constants::SecLevel::Info);

    while machine().should_loop_threads() {
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
pub fn do_coin_clean( c_date:&CDateT)
{
  // * remove from i_trx_utxo the the entries which are visible by blocks that are not "leave" any more and
  // * they are placed 4 level backward(or older) in history
  // * since it is a optional process to lightening DB loadness, it could be done for 8 level previous too

  let minimum_date:CDateT = application().get_now();
  // JSonObject leaves = LeavesHandler::getLeaveBlocks();
    /*
  StringList leaves_hashes = leaves.keys();
  for (CBlockHashT a_key: leaves_hashes)
    if (minimum_date > leaves[a_key].toObject().value("bCDate").to_string())
      minimum_date = leaves[a_key].toObject().value("bCDate").to_string();

  StringList ancestors = DAG::getAncestors(leaves_hashes, 8);
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
        CBlockHashT block_hash = row.value("b_hash").to_string();
        CLog::log("removing coins which are visible_by " + block_hash, "app", "trace");
        removeVisibleOutputsByBlocks(StringList{block_hash}, false);
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
    c_date = application().get_now();

  String full_query = "SELECT DISTINCT ut_visible_by, ut_creation_date FROM " + stbl_trx_utxos +
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
    CBlockHashT to_be_deleted_blcok = a_visibility_entry.value("ut_visible_by").to_string();
    if (!cutils::isValidHash(to_be_deleted_blcok))
    {
      CLog::log("Invalid block hash as to_be_deleted_blcok code! " + to_be_deleted_blcok, "sec", "fatal");
      return false;
    }
    StringList block_hashes = DAG::getDescendents(StringList{to_be_deleted_blcok});  // first generation of descendents
    block_hashes = cutils::arrayAdd(block_hashes, DAG::getDescendents(block_hashes));  // second generation
    block_hashes = cutils::arrayAdd(block_hashes, DAG::getDescendents(block_hashes));  // third generation
    block_hashes = cutils::arrayUnique(block_hashes);
    QVDRecordsT descendent_blocks = DAG::excludeFloatingBlocks(block_hashes); // exclude floating blocks
    CLog::log("visible_bys after exclude floating signature blocks: " + cutils::dumpIt(descendent_blocks), "trx", "trace");

    // avoid duplicate constraint error
    QueryRes candid_res = DbModel::select(
      stbl_trx_utxos,
      {"ut_coin"},
      {{"ut_visible_by", to_be_deleted_blcok}},
      {},
      0,
      false,
      false);
    StringList candid_coins = {};
    for(QVDicT a_coin: candid_res.records)
      candid_coins.push(a_coin.value("ut_coin").to_string());

    for (QVDicT a_descendent_block: descendent_blocks)
    {

      QueryRes existed_res = DbModel::select(
        stbl_trx_utxos,
        {"ut_coin"},
        {{"ut_visible_by", a_descendent_block.value("b_hash").to_string()}},
        {},
        0,
        false,
        false);

      StringList existed_coins {};
      for(QVDicT elm: existed_res.records)
        existed_coins.push(elm.value("ut_coin").to_string());

      StringList updateables = cutils::arrayDiff(candid_coins, existed_coins);
      if (updateables.len() > 0)
      {
        // TODO: improve it
        auto updateable_chunks = cutils::chunkStringList(updateables, 100);
        for (StringList a_chunk: updateable_chunks)
        {
          DbModel::update(
            stbl_trx_utxos,
            {{"ut_visible_by", a_descendent_block.value("b_hash").to_string()}},
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
        stbl_trx_utxos,
        {{"ut_visible_by", to_be_deleted_blcok}},
        true,
        false);

      removeCoinFromCachedSpendableCoins(to_be_deleted_blcok, "");
    }

  }

  return true;
}


ClausesT UTXOHandler::prepareUTXOQuery(
  const StringList& coins,
  const StringList& visible_by)
{
  ClausesT clauses = {};

  if (coins.len() > 0)
    clauses.push({"ut_coin", coins, "IN"});

  if (visible_by.len() > 0)
    clauses.push({"ut_visible_by", visible_by, "IN"});

  return clauses;
}

QVDRecordsT UTXOHandler::searchInSpendableCoinsCache(
  const StringList& coins)
{
  QVDRecordsT out {};
  auto[status, cachedSpendableCoins] = CMachine::cachedSpendableCoins();
  if (!status)
  {
    CLog::log("couldn't read from cached Spendable Coins!", "app", "fatal");
  }
  for (QVDicT a_coin: cachedSpendableCoins)
    if (coins.contains(a_coin.value("ut_coin").to_string()))
      out.push(a_coin);
  return out;
}

QVDRecordsT UTXOHandler::searchInSpendableCoins(
  const ClausesT& clauses,
  const StringList& fields,
  const OrderT& order,
  const uint64_t limit)
{
  String complete_query = "SELECT DISTINCT (ut_coin) ut_coin ";
  StringList complete_fields {"ut_coin"};

  if (fields.len() > 0)
  {
    complete_query += "," + fields.join(", ");
    for(String a_field: fields)
      complete_fields.push(a_field);
  }

  complete_query += " FROM  " + stbl_trx_utxos + " ";

  QueryElements qElms = DbModel::pre_query_generator(clauses, order);
  complete_query += qElms.m_clauses;

  if (limit > 0 )
    complete_query += " LIMIT " + String::number(limit);

  QueryRes res = DbModel::customQuery(
    "db_comen_spendable_coins",
    complete_query,
    complete_fields,
    0,
    qElms.m_values,
    false,
    false
  );

  return res.records;
}

/**
 *
 * @param {*} args function clons entire entries are visible_by given block(ancestors)
 * to new entries which are visible_by new-block
 */
void UTXOHandler::inheritAncestorsVisbility(
  const StringList& ancestor_blocks,
  const String& creation_date,
  const String& new_block_hash)
{
  // clog.trx.info(`inherit AncestorsVisbility: ${JSON.stringify(args)}`)
  // clog.trx.info(`ancestor_blocks==============================: ${ancestor_blocks}`)
  QueryRes currentVisibility = DbModel::select(
    stbl_trx_utxos,
    {"ut_coin", "ut_o_address", "ut_o_value", "ut_ref_creation_date"},
    {{"ut_visible_by", ancestor_blocks, "IN"}},
    {},
    0,
    false,
    false);
  // clog.trx.info(`currentVisibility: ${JSON.stringify(currentVisibility)}`);

  for (QVDicT a_coin: currentVisibility.records)
  {
    addNewUTXO(
      creation_date,
      a_coin.value("ut_coin").to_string(),
      new_block_hash,
      a_coin.value("ut_o_address").to_string(),
      a_coin.value("ut_o_value").toDouble(),
      a_coin.value("ut_ref_creation_date").to_string());
  }
}

bool UTXOHandler::addNewUTXO(
  const CDateT& creation_date,
  const CCoinCodeT& the_coin,
  const CBlockHashT visible_by,
  const CAddressT& address,
  const CMPAISValueT& coin_value,
  const CDateT& coin_creation_date)
{
  if (!cutils::isValidHash(visible_by))
  {
    CLog::log("Invalid block hash as visibility code! " + visible_by, "sec", "fatal");
    return false;
  }

  if ((coin_value < 0) || (coin_value > MAX_COIN_VALUE))
  {
    CLog::log("Invalid coin value to insert! " + String::number(coin_value), "sec", "fatal");
    return false;
  }

  {
    // remove this code block after implementing bloom filter
    // TODO: implement bloom filter in order to avoid select and reduce the db load ASAP
    if (CMachine::cachedCoinsVisibility().records.len() == 0)
      assignCacheCoinsVisibility();

    if (CMachine::cachedCoinsVisibility().records.len() == 0)
    {
      // why? hwo it dissapeared? BTW re-assign it
      assignCacheCoinsVisibility();
    }
    if (CMachine::cachedCoinsVisibility("contains", {the_coin + visible_by}).is_visible)
      return true;
  }

  QueryRes dblChk = DbModel::select(
    stbl_trx_utxos,
    {"ut_coin"},
    {{"ut_coin", the_coin},
    {"ut_visible_by", visible_by}},
    {},
    0,
    true,
    false);

  if (dblChk.records.len() > 0)
  {
//    CLog::log("The coin already imported. coin(" + cutils::shortCoinRef(the_coin)+ ") already is visible by block(" + cutils::hash8c(visible_by)+ ")", "trx", "trace");
    return true;
  }

  // clog.trx.info(`add NewUTXO maturated block(${utils.hash6c(args.visible_by)}) cycle/cloneCode: ${args.cloneCode} ${utils.hash8c(args.address)} ${utils.microPAIToPAI(args.value)} pai`);
  QVDicT values {
    {"ut_creation_date", creation_date},
    {"ut_coin", the_coin},
    {"ut_visible_by", visible_by},
    {"ut_o_address", address},
    {"ut_o_value", QVariant::fromValue(coin_value).toDouble()},
    {"ut_ref_creation_date", coin_creation_date}
  };
  bool res = DbModel::insert(
    stbl_trx_utxos,
    values,
    true,
    false);

  CMachine::cachedCoinsVisibility("append", {the_coin + visible_by});

  CMachine::cachedSpendableCoins("append", {values});
  return res;
}

/**
 *
 * @param {*} hashes
 * Only used for mitigate table load
 */
bool UTXOHandler::removeVisibleOutputsByBlocks(const StringList& block_hashes, const bool do_control)
{
  StringList unremoveable_blocks = {};
  StringList unremoveable_coins = {};

  for (CBlockHashT a_block: block_hashes)
  {
    if (!do_control)
    {
      DbModel::dDelete(
        stbl_trx_utxos,
        {{"ut_visible_by", a_block}},
        false,
        false);
      continue;
    }


    QueryRes removing_candidates = DbModel::select(
      stbl_trx_utxos,
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
        stbl_trx_utxos,
        {"ut_coin", "ut_visible_by"},
        {{"ut_coin", a_coin.value("ut_coin")},
        {"ut_creation_date", a_coin.value("ut_creation_date"), ">"}},
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
        unremoveable_coins.push(a_coin.value("ut_coin").to_string());
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
        StringList all_ancestors_of_a_younger_block = DAG::returnAncestorsYoungerThan(
          {a_visibility.value("ut_visible_by").to_string()},
          a_coin.value("ut_creation_date").to_string());

        if (all_ancestors_of_a_younger_block.len() == 0)
          continue;

        if (all_ancestors_of_a_younger_block.contains(a_coin.value("ut_visible_by").to_string()))
          is_visible_by_ancestors = true;
      }
      if (!is_visible_by_ancestors)
      {
        // security issue
        String msg = "The ut_coin which want to remove does not exist in history of newer entries! a_coin: " + cutils::dumpIt(a_coin) + " younger Visibility Of RefLoc: " + cutils::dumpIt(younger_visibility_of_coins.records);
        CLog::log(msg, "sec", "error");
        unremoveable_blocks.push(a_block);
        unremoveable_coins.push(a_coin.value("ut_coin").to_string());
        continue;
      }

      // finally remove utxo which is visible by his descendents
      if (!unremoveable_blocks.contains(a_block))
      {
        DbModel::dDelete(
          stbl_trx_utxos,
          {{"ut_visible_by", a_block},
          {"ut_coin", a_coin.value("ut_coin").to_string()}},
          true,
          false);

        removeCoinFromCachedSpendableCoins(a_block, "");
      }

    }
  }

  if ( (unremoveable_blocks.len() > 0) || (unremoveable_coins.len() > 0) )
  {
    unremoveable_blocks = cutils::arrayUnique(unremoveable_blocks);
    unremoveable_coins = cutils::arrayUnique(unremoveable_coins);
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
  auto [status, current_cache] = CMachine::cachedSpendableCoins("remove", {}, visible_by, the_coin);
}

bool UTXOHandler::removeCoin(const CCoinCodeT& the_coin)
{
//  CLog::log("remove an spent coin(" + cutils::shortCoinRef(the_coin) + ")", "trx", "trace");
  DbModel::dDelete(
    stbl_trx_utxos,
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
      removeCoin(input->getCoinCode());
  return true;
}

std::tuple<CMPAIValueT, QVDRecordsT, QV2DicT> UTXOHandler::getSpendablesInfo()
{
  QueryRes res = DbModel::select(
    stbl_trx_utxos,
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
    CCoinCodeT the_coin = a_coin.value("ut_coin").to_string();
    if (!utxos_dict.keys().contains(the_coin))
    {
      utxos_dict[the_coin] = QVDicT {
        {"refLocCreationDate", a_coin.value("ut_ref_creation_date").to_string()},
        {"outValue", a_coin.value("ut_o_value").toDouble()},
        {"visibleBy", StringList{}}};

      utxos.push(QVDicT {
        {"refLoc", the_coin},
        {"refLocCreationDate", a_coin.value("ut_ref_creation_date").to_string()},
        {"outValue", a_coin.value("ut_o_value").toDouble()}});

      sum += a_coin.value("ut_o_value").toDouble();
    }
    auto tmp = utxos_dict[the_coin]["visibleBy"].toList();
    tmp.push(a_coin.value("ut_visible_by"));
    utxos_dict[the_coin]["visibleBy"] = tmp;
  }
  return {sum, utxos, utxos_dict };
}

QVDRecordsT UTXOHandler::extractUTXOsBYAddresses(const StringList& addresses)
{
  if (addresses.len() == 0)
    return {};

  auto[clauses_, values_] = DbModel::clauses_query_generator({{"ut_o_address", addresses, "IN"}});

  String complete_query = "SELECT ut_coin, ut_o_address, ut_o_value, min(ut_ref_creation_date) AS ref_creation_date ";
  complete_query += "FROM " + stbl_trx_utxos + " WHERE " + clauses_;
  complete_query += "GROUP BY ut_coin, ut_o_address, ut_o_value ORDER BY min(ut_ref_creation_date), ut_o_address, ut_o_value";

  QueryRes utxos = DbModel::customQuery(
    "db_comen_spendable_coins",
    complete_query,
    {"ut_coin", "ut_o_address", "ut_o_value", "ref_creation_date"},
    0,
    values_,
    true,
    false);
  QVDRecordsT new_UTXOs = {};
  for (QVDicT element: utxos.records)
  {
    new_UTXOs.push(QVDicT {
      {"ut_ref_creation_date", element.value("ref_creation_date")},
      {"ut_coin", element.value("ut_coin")},
      {"ut_o_address", element.value("ut_o_address")},
      {"ut_o_value", element.value("ut_o_value")}});
  };
  return new_UTXOs;
}

QVDRecordsT UTXOHandler::generateCoinsVisibilityReport()
{
  String complete_query = "SELECT DISTINCT ut_visible_by, ut_coin, ut_o_address, ut_o_value FROM " + stbl_trx_utxos + " ORDER BY ut_visible_by, ut_coin, ut_o_address, ut_o_value";
  QueryRes utxos = DbModel::customQuery(
    "db_comen_spendable_coins",
    complete_query,
    {"ut_visible_by", "ut_coin", "ut_o_address", "ut_o_value"},
    0,
    {},
    true,
    false);

  QV2DicT visibility = {};
  QHash<CBlockHashT, std::vector<String> > coins_list = {};
  QHash<CBlockHashT, std::vector<String> > owners_list = {};
  QHash<CCoinCodeT, CMPAIValueT> map_coins_to_value = {};

  for (QVDicT a_row: utxos.records)
  {
    CBlockHashT visible_by = a_row.value("ut_visible_by").to_string();
    CCoinCodeT the_coin = a_row.value("ut_coin").to_string();
    CAddressT address = a_row.value("ut_o_address").to_string();
    CMPAIValueT coin_value = a_row.value("ut_o_value").toDouble();

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
  StringList blocks_hashes = coins_list.keys();
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
      blocks_dict[b.value("b_hash").to_string()] = QVDicT {
        {"block_type", b.value("b_type").to_string()},
        {"block_creation_date", b.value("b_creation_date").to_string()}};
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
  StringList keys = visibility.keys();
  keys.sort();
  for(String key: keys)
    out.push(visibility[key]);

  return out;
}

void UTXOHandler::assignCacheCoinsVisibility()
{
  QueryRes existed = DbModel::select(
    stbl_trx_utxos,
    {"ut_coin", "ut_visible_by"});
  String tmp_vis_coin = "";
  for (QVDicT elm: existed.records)
  {
    tmp_vis_coin = elm.value("ut_coin").to_string() + elm.value("ut_visible_by").to_string();
    CMachine::cachedCoinsVisibility("append", {tmp_vis_coin});
  }

}

 */