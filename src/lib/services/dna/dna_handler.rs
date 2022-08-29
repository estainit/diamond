use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, cutils, dlog, machine};
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{CAddressT, CDateT, SharesCountT, SharesPercentT};
use crate::lib::database::abs_psql::{q_custom_query, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::C_SHARES;

#[derive(Debug)]
pub struct Shareholder {
    #[allow(unused, dead_code)]
    account: CAddressT,
    #[allow(unused, dead_code)]
    shares: SharesCountT,
}

/*

  CDocHashT m_project_hash = "";
  CAddressT m_shareholder = "";
  uint64_t m_help_hours = 0;
  uint64_t m_help_level = 0;
  DNAShareCountT m_shares = 0;
  uint64_t m_votes_yes = 0;
  uint64_t m_votes_abstain = 0;
  uint64_t m_votes_no = 0;


*/

//old_name_was insertAShare
pub fn insert_a_share(doc: &Document) -> (bool, String)
{
    let single_value = doc.get_doc_hash().clone();
    let (_status, records) = q_select(
        C_SHARES,
        vec!["dn_doc_hash"],
        vec![simple_eq_clause("dn_doc_hash", &single_value)],
        vec![],
        1,
        false);
    if records.len() > 0
    {
        // maybe some updates
        return (false, "share already exist!".to_string()); // "The DNA document (${utils.hash6c(dna.hash)}) is already recorded"};
    }

    if application().is_greater_than_now(&doc.m_doc_creation_date)
    {
        return (false, format!("share is newer than now! {}", doc.m_doc_creation_date));
    }

    let dn_help_hours = doc.m_if_proposal_doc.m_help_hours;
    let dn_help_level = doc.m_if_proposal_doc.m_help_level;
    let dn_shares = doc.m_if_proposal_doc.m_shares;
    let dn_votes_y= doc.m_if_proposal_doc.m_votes_yes;
    let dn_votes_a= doc.m_if_proposal_doc.m_votes_abstain;
    let dn_votes_n= doc.m_if_proposal_doc.m_votes_no;

    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("dn_doc_hash", &doc.m_doc_hash as &(dyn ToSql + Sync)),
        ("dn_shareholder", &doc.m_if_proposal_doc.m_contributor_account as &(dyn ToSql + Sync)),
        ("dn_project_hash", &doc.m_if_proposal_doc.m_project_hash as &(dyn ToSql + Sync)),
        ("dn_help_hours", &dn_help_hours as &(dyn ToSql + Sync)),
        ("dn_help_level", &dn_help_level as &(dyn ToSql + Sync)),
        ("dn_shares", &dn_shares as &(dyn ToSql + Sync)),
        ("dn_title", &doc.m_doc_title as &(dyn ToSql + Sync)),
        ("dn_descriptions", &doc.m_doc_comment as &(dyn ToSql + Sync)),
        ("dn_tags", &doc.m_doc_tags as &(dyn ToSql + Sync)),
        ("dn_votes_y", &dn_votes_y as &(dyn ToSql + Sync)),
        ("dn_votes_a", &dn_votes_a as &(dyn ToSql + Sync)),
        ("dn_votes_n", &dn_votes_n as &(dyn ToSql + Sync)),
        ("dn_creation_date", &doc.m_doc_creation_date as &(dyn ToSql + Sync))
    ]);

    dlog(
        &format!("Inserting a DNA share: {:?}", &values),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    q_insert(
        C_SHARES,    // table
        &values, // values to insert
        true,
    );

    return (true, "the share was inserted".to_string());
}

/*
GenRes DNAHandler::insertAShare(JSonObject& params)
{
  QueryRes exist = DbModel::select(
    DNAHandler::stbl_dna_shares,
    StringList {"dn_doc_hash"},     // fields
    {ModelClause("dn_doc_hash", params["dn_doc_hash"].to_string())}
    );
  if (exist.records.len() > 0)
  {
    // maybe some updates
    return {false, "The DNA document (${utils.hash6c(dna.hash)}) is already recorded"};
  }

//  cutils::exitIfGreaterThanNow(params["m_doc_creation_date"].to_string());

  QVDicT values{
    {"dn_doc_hash", params["m_doc_hash"].to_string()},
    {"dn_shareholder", params["m_shareholder"].to_string()},
    {"dn_project_hash", params["m_project_hash"].to_string()},
    {"dn_help_hours", params["m_help_hours"].to_string()},
    {"dn_help_level", params["m_help_level"].to_string()},
    {"dn_shares", params["m_shares"].to_string()},
    {"dn_title", params["m_doc_title"].to_string()},
    {"dn_descriptions", params["m_doc_comment"].to_string()},
    {"dn_tags", params["m_doc_tags"].to_string()},
    {"dn_votes_y", params["m_votes_yes"].to_string()},
    {"dn_votes_a", params["m_votes_abstain"].to_string()},
    {"dn_votes_n", params["m_votes_no"].to_string()},
    {"dn_creation_date", params["m_block_creation_date"].to_string()}
  };

  DbModel::insert(
    stbl_dna_shares,    // table
    values, // values to insert
    true
  );

  return {true, ""};
}

/**
 *
 * @param {*} _t
 * given time(DNA proposal approing time), it returns the range in which a share is valid
 * the active period starts from 7 years ago and finishes right at the end of previous cycle time
 * it means if your proposal have been approved in 2017-01-01 00:00:00, the owner can gain from 2017-01-01 12:00:00 to 2024-01-01 00:00:00
 */

 */
//old_name_was getDNAActiveDateRange
pub fn get_dna_active_date_range(c_date: &CDateT) -> (String, String)
{
    let mut the_range = application().get_a_cycle_range(
        c_date,
        constants::SHARE_MATURITY_CYCLE,
        0);

    // panic!("mmmmmm the r4 ange: {:?}", the_range);

    if application().cycle_length() == 1
    {
        the_range.from = application().years_before(
            constants::CONTRIBUTION_APPRECIATING_PERIOD as u64,
            &the_range.from);
    } else {
        let back_in_time = 100 * application().get_cycle_by_minutes();
        the_range.from = application().minutes_before(
            back_in_time,
            &the_range.from);
    }
    return (
        the_range.from.clone(),
        the_range.to.clone());
}

// TODO: since shares are counting for before 2 last cycles, so implementing a caching system will be much helpfull where we have millions of shareholders
//old_name_was getSharesInfo
pub fn get_shares_info(c_date: &CDateT) -> (SharesCountT, HashMap<String, SharesCountT>, Vec<Shareholder>)
{
    // cDate = application().now();

    // retrieve the total shares in last 24 hours, means -36 to -24 based on greenwich time
    // (Note: it is not the machine local time)
    // for examplie if a node runs this command on 7 May at 14 (in greenwich time)
    // the result will be the final state of DNA at 11:59:59 of 6 May.
    // it means the node calculate all shares which the creation date are less than 11:59:59  of 6 May
    // -------------< 11:59 of 6 May |         --- 24 hours ---        |12:00 of 7 May     --- 2 hours ---     14:00 of 7 May

    dlog(
        &format!("get Share info: calc shares for date({})", c_date),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    let (min_creation_date, max_creation_date) = get_dna_active_date_range(c_date);
    dlog(
        &format!("get Share info res: calc shares for min_creation_date({}) max_creation_date({})", min_creation_date, max_creation_date),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    let mut query = "".to_string();
    if constants::DATABASAE_AGENT == "psql"
    {
        query = "SELECT dn_shareholder, CAST(SUM(dn_shares) AS varchar) AS shares_amount FROM ".to_owned() + C_SHARES;
        query += &*(" WHERE dn_creation_date between '".to_owned() + &min_creation_date + &"' AND '".to_owned() + &max_creation_date + "' GROUP BY dn_shareholder ORDER BY shares_amount DESC");
    } else if constants::DATABASAE_AGENT == "sqlite"
    {
        query = "SELECT dn_shareholder, SUM(dn_shares) sum_ FROM ".to_owned() + C_SHARES;
        query += &*(" WHERE dn_creation_date between \"".to_owned() + &min_creation_date + &"\" AND \"".to_owned() + &max_creation_date + "\" GROUP BY dn_shareholder ORDER BY sum_ DESC");
    }
    dlog(
        &format!("Retrieve shares for range cDate({}) -> ({}, {})", c_date, min_creation_date, max_creation_date),
        constants::Modules::App,
        constants::SecLevel::Info);

    // let msg = `Retrieve shares: SELECT shareholder _shareholder, SUM(shares) _share FROM i_dna_shares WHERE creation_date between '${minCreationDate}' AND '${maxCreationDate}' GROUP BY _shareholder ORDER BY _share DESC `;
    let (_status, records) = q_custom_query(
        &query,
        &vec![],
        true);

    let mut sum_shares: SharesCountT = 0.0;
    let mut holders_order_by_shares: Vec<Shareholder> = vec![];
    let mut share_amount_per_holder: HashMap<String, SharesCountT> = HashMap::new();
    for a_share in records
    {
        let sum_: f64 = a_share["shares_amount"].parse::<f64>().unwrap();
        sum_shares += sum_;
        let owner: CAddressT = a_share["dn_shareholder"].to_string();
        share_amount_per_holder.insert(owner.clone(), sum_);
        holders_order_by_shares.push(
            Shareholder {
                account: owner.clone(),
                shares: share_amount_per_holder[&owner],
            });
    }
    dlog(
        &format!("1. sum_shares: {:?}", &sum_shares),
        constants::Modules::App,
        constants::SecLevel::Debug);
    dlog(
        &format!("2. share_amount_per_holder: {:?}", &share_amount_per_holder),
        constants::Modules::App,
        constants::SecLevel::Debug);
    dlog(
        &format!("3. holders_order_by_shares: {:?}", &holders_order_by_shares),
        constants::Modules::App,
        constants::SecLevel::Debug);

    return (sum_shares, share_amount_per_holder, holders_order_by_shares);
}

/*
std::tuple<DNAShareCountT, DNASharePercentT> DNAHandler::getAnAddressShares(
  const CAddressT& address,
  CDateT cDate)
{
  if(cDate == "")
    cDate = application().now();

  auto[sum_shares, share_amount_per_holder, tmp_] = getSharesInfo(cDate);
  Q_UNUSED(tmp_);
  DNAShareCountT shares = 0.0;
  double percentage = 0.0;
  if (share_amount_per_holder.keys().contains(address))
  {
    shares = share_amount_per_holder[address];
    percentage = ((shares * 100) / sum_shares);
  }
  percentage = cutils::iFloorFloat(percentage);
  return {shares, percentage};
}

*/
//old_name_was getMachineShares
pub fn get_machine_shares(c_date: &CDateT) -> (String, SharesCountT, SharesPercentT)
{
    let (sum_shares, share_amount_per_holder, _tmp) = get_shares_info(c_date);
    let backer_address: CAddressT = machine().get_backer_address();
    let mut shares: SharesCountT = 0.0;
    if share_amount_per_holder.contains_key(&*backer_address) {
        shares = share_amount_per_holder[&backer_address];
    }
    let percentage: SharesPercentT = cutils::i_floor_float((shares * 100.0) / sum_shares);
    return (backer_address, shares, percentage);
}
/*

QVDRecordsT DNAHandler::searchInDNA(
  const ClausesT& clauses,
  const StringList& fields,
  const OrderT order,
  const uint64_t limit)
{
  QueryRes res = DbModel::select(
    stbl_dna_shares,
    fields,
    clauses,
    order,
    limit);
  return res.records;
}

void DNAHandler::updateDNAVotes(
  const ClausesT& clauses,
  const QVDicT& updates)
{
  DbModel::update(
    stbl_dna_shares,
    updates,
    clauses);
}

*/