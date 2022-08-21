use crate::{constants, cutils, dlog};
use crate::cutils::TimeRange;
use crate::lib::custom_types::{CDateT, CMPAIValueT};
use crate::lib::database::abs_psql::q_custom_query;
use crate::lib::database::tables::C_TREASURY;

//old_name_was getTreasureIncomesDateRange
pub fn get_treasure_incomes_date_range(c_date: &CDateT) -> TimeRange
{
    return cutils::get_a_cycle_range(c_date, constants::TREASURY_MATURATION_CYCLES, 0);
}

//old_name_was calcTreasuryIncomes
pub fn calc_treasury_incomes(c_date: &CDateT) -> (String, String, CMPAIValueT)
{

    // retrieve the total treasury incomes in last cycle (same as share cycle calculation)
    let the_range = get_treasure_incomes_date_range(c_date);

    let mut complete_query: String = "".to_string();
    if constants::DATABASAE_AGENT == "psql"
    {
        complete_query = "SELECT CAST(SUM(tr_value) AS varchar) AS incomes_amount FROM ".to_owned() + C_TREASURY + " WHERE tr_creation_date between '" + &*the_range.from + "' AND '" + &*the_range.to + "' ";
        // complete_query = "SELECT tr_value AS incomes_amount FROM ".to_owned() + STBL_TREASURY + " WHERE tr_creation_date between '" + &*the_range.from + "' AND '" + &*the_range.to + "' ";
    } else if constants::DATABASAE_AGENT == "sqlite"
    {
        complete_query = "SELECT SUM(tr_value) incomes_amount FROM ".to_owned() + C_TREASURY + " WHERE tr_creation_date between \"" + &*the_range.from + "\" AND \"" + &*the_range.to + "\" ";
    }

    let (_status, records) = q_custom_query(
        &complete_query,
        &vec![],
        true);
    dlog(
        &format!("calc Treasury Incomes WHERE creation_date between ({},{}) -> incomes: {:?}",
                 the_range.from, the_range.to, &records),
        constants::Modules::CB,
        constants::SecLevel::Info);

    let mut income_value: CMPAIValueT = 0;
    if records[0]["incomes_amount"] != "" {
        if records[0]["incomes_amount"].parse::<CMPAIValueT>().unwrap() > 0
        {
            income_value = records[0]["incomes_amount"].parse::<CMPAIValueT>().unwrap();
        }
    }

    return (
        the_range.from.clone(),
        the_range.to.clone(),
        income_value
    );
}
/*

void TreasuryHandler::insertIncome(
  String title,
  String category,
  String descriptions,
  String creation_date,
  CMPAIValueT value,
  String block_hash,
  CCoinCodeT coin)
{
  QueryRes dbl = DbModel::select(
    STBL_TREASURY,
    {"tr_coin"},
    {{"tr_coin", coin}},
    {{"tr_id", "ASC"}});
  if (dbl.records.len() > 0)
  {
    CLog::log("duplicated treasury insertion: block(" + cutils::hash8c(block_hash)+ ") title(" + title + ")", "trx", "warning");
    // update the descriptions
    DbModel::update(
      STBL_TREASURY,
      {{"tr_descriptions", dbl.records[0].value("tr_descriptions").to_string() + " " + descriptions}},
      {{"tr_coin", coin}});
    return;
  }

  QVDicT values {
    {"tr_cat", category},
    {"tr_title", title},
    {"tr_descriptions", descriptions},
    {"tr_creation_date", creation_date},
    {"tr_block_hash", block_hash},
    {"tr_coin", coin},
    {"tr_value", QVariant::fromValue(value)}};

  CLog::log("Treasury income(" + cutils::microPAIToPAI6(value) + " PAIs) because of block(" + cutils::hash8c(block_hash)+ ") title(" + title + ") values :" + cutils::dumpIt(values ), "trx", "info");

  DbModel::insert(
    STBL_TREASURY,
    values);

  return;
}

void TreasuryHandler::donateTransactionInput(
  String title,
  String category,
  String descriptions,
  String creation_date,
  CMPAIValueT value,
  String block_hash,
  CCoinCodeT coin)
{

  if (title == "")
    title = "No Title!";

  if (category == "")
    category = "No category!";

  if (descriptions == "")
    descriptions = "No descriptions!";

  if (creation_date == "")
    creation_date = "No creation_date!";

  // retrieve location refLoc is generated
  // let blocks = dagHandler.retrieveBlocksInWhichARefLocHaveBeenProduced(args.refLoc);
  // clog.trx.info(`donate Transaction Input. blocks by refLoc: ${JSON.stringify(blocks)}`);
  // let block = blocks[0];

  // big FIXME: for cloning transactions issue
  insertIncome(
    title,
    category,
    descriptions,
    creation_date,
    value,
    block_hash,
    coin);
}

CMPAIValueT TreasuryHandler::getWaitedIncomes(CDateT cDate)
{
  if (cDate == "")
    cDate = cutils::getNow();

  cDate = getTreasureIncomesDateRange(cDate).to;

  QueryRes res = DbModel::select(
    STBL_TREASURY,
    {"tr_value"},
    {{"tr_creation_date", cDate, ">"}});

  CMPAIValueT sum = 0;
  for (QVDicT income: res.records)
    sum += income.value("tr_value").toDouble();

  return sum;
}

QVDRecordsT TreasuryHandler::searchInTreasury(
  const ClausesT& clauses,
  const StringList& fields,
  const OrderT order,
  const uint64_t limit)
{
  QueryRes res = DbModel::select(
    STBL_TREASURY,
    fields,
    clauses,
    order,
    limit);
  return res.records;
}

*/