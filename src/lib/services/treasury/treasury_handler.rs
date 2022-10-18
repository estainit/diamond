use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, cutils, dlog};
use crate::lib::custom_types::{CCoinCodeT, CDateT, CMPAISValueT, CMPAIValueT};
use crate::lib::database::abs_psql::{OrderModifier, q_custom_query, q_insert, q_select, q_update, simple_eq_clause};
use crate::lib::database::tables::C_TREASURY;
use crate::lib::machine::app_params::TimeRange;

//old_name_was getTreasureIncomesDateRange
pub fn get_treasure_incomes_date_range(c_date: &CDateT) -> TimeRange
{
    return application().get_a_cycle_range(c_date, constants::TREASURY_MATURATION_CYCLES, 0);
}

//old_name_was calcTreasuryIncomes
pub fn calc_treasury_incomes(c_date: &CDateT) -> (String, String, CMPAIValueT)
{

    // retrieve the total treasury incomes in last cycle (same as share cycle calculation)
    let the_range = get_treasure_incomes_date_range(c_date);

    let mut complete_query: String = "".to_string();
    if constants::DATA_BASAE_AGENT == "psql"
    {
        complete_query = "SELECT CAST(SUM(tr_value) AS varchar) AS incomes_amount FROM ".to_owned() + C_TREASURY + " WHERE tr_creation_date between '" + &*the_range.from + "' AND '" + &*the_range.to + "' ";
        // complete_query = "SELECT tr_value AS incomes_amount FROM ".to_owned() + C_TREASURY + " WHERE tr_creation_date between '" + &*the_range.from + "' AND '" + &*the_range.to + "' ";
    } else if constants::DATA_BASAE_AGENT == "sqlite"
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

//old_name_was insertIncome
pub fn insert_income(
    title: &String,
    category: &String,
    descriptions: &String,
    creation_date: &String,
    income_amount: CMPAIValueT,
    block_hash: &String,
    coin: &CCoinCodeT)
{
    let (_status, dbl) = q_select(
        C_TREASURY,
        vec!["tr_coin"],
        vec![simple_eq_clause("tr_coin", coin)],
        vec![
            &OrderModifier { m_field: "tr_id", m_order: "ASC" },
        ],
        0,
        true);
    if dbl.len() > 0
    {
        dlog(
            &format!(
                "Duplicated treasury insertion: block({}) title({})",
                cutils::hash8c(block_hash),
                title),
            constants::Modules::Trx,
            constants::SecLevel::Warning);
        // update the descriptions
        let tr_descriptions = dbl[0]["tr_descriptions"].to_string() + " " + descriptions;
        let update_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("tr_descriptions", &tr_descriptions as &(dyn ToSql + Sync))
        ]);
        q_update(
            C_TREASURY,
            &update_values,
            vec![simple_eq_clause("tr_coin", coin)],
            true);
        return;
    }
    let income_amount = income_amount as CMPAISValueT;
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("tr_cat", &category as &(dyn ToSql + Sync)),
        ("tr_title", &title as &(dyn ToSql + Sync)),
        ("tr_descriptions", &descriptions as &(dyn ToSql + Sync)),
        ("tr_creation_date", &creation_date as &(dyn ToSql + Sync)),
        ("tr_block_hash", &block_hash as &(dyn ToSql + Sync)),
        ("tr_coin", &coin as &(dyn ToSql + Sync)),
        ("tr_value", &income_amount as &(dyn ToSql + Sync)),
    ]);

    dlog(
        &format!(
            "Treasury income({} Ray) because of block({}) title({}) values: {:?}",
            cutils::nano_pai_to_pai(income_amount),
            cutils::hash8c(&block_hash),
            title,
            values
        ),
        constants::Modules::Trx,
        constants::SecLevel::Info);
    q_insert(
        C_TREASURY,
        &values,
        true);

    return;
}

//old_name_was donateTransactionInput
pub fn donate_transaction_input(
    title: &String,
    category: &String,
    descriptions: &String,
    creation_date: &String,
    value: CMPAIValueT,
    block_hash: &String,
    coin: &CCoinCodeT)
{
    let mut title = title.clone();
    let mut category = category.clone();
    let mut descriptions = descriptions.clone();
    let mut creation_date = creation_date.clone();

    if title == ""
    { title = "No Title!".to_string(); }

    if category == ""
    { category = "No category!".to_string(); }

    if descriptions == ""
    { descriptions = "No descriptions!".to_string(); }

    if creation_date == ""
    { creation_date = "No creation_date!".to_string(); }

    // retrieve location refLoc is generated
    // let blocks = retrieve_blocks_in_which_a_coin_have_been_produced(args.refLoc);
    // clog.trx.info(`donate Transaction Input. blocks by refLoc: ${JSON.stringify(blocks)}`);
    // let block = blocks[0];

    // big FIXME: for cloning transactions issue
    insert_income(
        &title,
        &category,
        &descriptions,
        &creation_date,
        value,
        block_hash,
        coin);
}

/*
CMPAIValueT TreasuryHandler::getWaitedIncomes(CDateT cDate)
{
  if (cDate == "")
    cDate = cutils::getNow();

  cDate = getTreasureIncomesDateRange(cDate).to;

  QueryRes res = DbModel::select(
    C_TREASURY,
    {"tr_value"},
    {{"tr_creation_date", cDate, ">"}});

  CMPAIValueT sum = 0;
  for (QVDicT income: res.records)
    sum += income["tr_value"].toDouble();

  return sum;
}

QVDRecordsT TreasuryHandler::searchInTreasury(
  const ClausesT& clauses,
  const VString& fields,
  const OrderT order,
  const uint64_t limit)
{
  QueryRes res = DbModel::select(
    C_TREASURY,
    fields,
    clauses,
    order,
    limit);
  return res.records;
}

*/