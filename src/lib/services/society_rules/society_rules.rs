use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, ccrypto, constants, cutils, dlog};
use crate::lib::custom_types::{BlockLenT, CDateT, CMPAIValueT, DocLenT, SharesPercentT};
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::C_ADMINISTRATIVE_REFINES_HISTORY;

pub mod polling_types
{
    pub const REQUEST_FOR_REFINE_BASE_PRICE: &str = "RFRfBasePrice";
    // char base price
    pub const REQUEST_FOR_REFINE_TX_PRICE: &str = "RFRfTxBPrice";
    pub const REQUEST_FOR_REFINE_BLOCK_FIXED_COST: &str = "RFRfBlockFixCost";
    pub const REQUEST_FOR_REFINE_POLLING_PRICE: &str = "RFRfPollingPrice";
    pub const REQUEST_FOR_REFINE_PLEDGE_PRICE: &str = "RFRfPLedgePrice";
    pub const REQUEST_FOR_REFINE_CONCLUDE_PLEDGE_PRICE: &str = "RFRfClPLedgePrice";
    pub const REQUEST_FOR_REFINE_PROPSAL_PRICE: &str = "RFRfDNAPropPrice";
    pub const REQUEST_FOR_REFINE_BALLOT_PRICE: &str = "RFRfBallotPrice";
    pub const REQUEST_FOR_REFINE_INAME_REGISTER_PRICE: &str = "RFRfINameRegPrice";
    pub const REQUEST_FOR_REFINE_INAME_BIND_PGP_PRICE: &str = "RFRfINameBndPGPPrice";
    pub const REQUEST_FOR_REFINE_INAME_MESSAGE_PRICE: &str = "RFRfINameMsgPrice";
    pub const REQUEST_FOR_REFINE_FREE_POST_PRICE: &str = "RFRfFPostPrice";
    pub const REQUEST_FOR_REFINE_MIN_SHARES_TO_WIKI: &str = "RFRfMinS2Wk";
    pub const REQUEST_FOR_REFINE_MIN_SHARES_TO_FORUMS: &str = "RFRfMinS2DA";
    pub const REQUEST_FOR_REFINE_MIN_SHARES_TO_VOTE: &str = "RFRfMinS2V";
    pub const REQUEST_FOR_REFINE_MIN_SHARES_TO_SIGN_BLOCKS: &str = "RFRfMinFSign";
    pub const REQUEST_FOR_REFINE_MIN_SHARES_TO_FLOATING_VOTES: &str = "RFRfMinFVote";
}

//old_name_was getAdmDefaultValues
#[allow(unused, dead_code)]
pub fn get_administrative_default_values() -> HashMap<String, f64>
{

    //minimum cost(100 trx * 2 PAI per trx * 1000000 micropPAI) for atleast 100 simple/light transaction
    let block_fix_cost: f64;
    if application().cycle_length() == 1
    {
        block_fix_cost = 100.0 * 2.0 * 1_000_000.0;
    } else {
        block_fix_cost = 1000.0;
    }

    let administrative_costs_params: HashMap<String, f64> = HashMap::from([
        (polling_types::REQUEST_FOR_REFINE_PLEDGE_PRICE.to_string(), 37.0),
        (polling_types::REQUEST_FOR_REFINE_POLLING_PRICE.to_string(), 37.0),
        (polling_types::REQUEST_FOR_REFINE_TX_PRICE.to_string(), 11.0),
        (polling_types::REQUEST_FOR_REFINE_CONCLUDE_PLEDGE_PRICE.to_string(), 37.0),
        (polling_types::REQUEST_FOR_REFINE_PROPSAL_PRICE.to_string(), 37.0),
        (polling_types::REQUEST_FOR_REFINE_BALLOT_PRICE.to_string(), 57.0),
        (polling_types::REQUEST_FOR_REFINE_INAME_REGISTER_PRICE.to_string(), 71.0),
        (polling_types::REQUEST_FOR_REFINE_INAME_BIND_PGP_PRICE.to_string(), 41.0),
        (polling_types::REQUEST_FOR_REFINE_INAME_MESSAGE_PRICE.to_string(), 11.0),
        (polling_types::REQUEST_FOR_REFINE_FREE_POST_PRICE.to_string(), 17.0),
        (polling_types::REQUEST_FOR_REFINE_BASE_PRICE.to_string(), 400.0),
        (polling_types::REQUEST_FOR_REFINE_BLOCK_FIXED_COST.to_string(), block_fix_cost),
        (polling_types::REQUEST_FOR_REFINE_MIN_SHARES_TO_WIKI.to_string(), 0.0001),        // create/edit wiki page
        (polling_types::REQUEST_FOR_REFINE_MIN_SHARES_TO_FORUMS.to_string(), 0.00001),       // create/edit Demos
        (polling_types::REQUEST_FOR_REFINE_MIN_SHARES_TO_VOTE.to_string(), 0.0000000001),  // participate in pollings and vote
        (polling_types::REQUEST_FOR_REFINE_MIN_SHARES_TO_SIGN_BLOCKS.to_string(), 0.0000000001),
        (polling_types::REQUEST_FOR_REFINE_MIN_SHARES_TO_FLOATING_VOTES.to_string(), 0.000000002)
        // ['RFRlRsCoins', {}],
    ]);

    return administrative_costs_params;
}

//old_name_was initAdministrativeConfigurationsHistory
pub fn init_administrative_configurations_history()
{
    let admin_cost_params: HashMap<String, f64> = get_administrative_default_values();
    let launch_date = application().launch_date();
    for (a_key, a_value) in admin_cost_params
    {
        let arh_hash: String = ccrypto::keccak256(&(launch_date.clone() + "-" + &a_key));
        let arh_value = cutils::convert_float_to_string(a_value, constants::FLOAT_LENGTH);
        let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("arh_hash", &arh_hash as &(dyn ToSql + Sync)),
            ("arh_subject", &a_key as &(dyn ToSql + Sync)),
            ("arh_value", &arh_value as &(dyn ToSql + Sync)),
            ("arh_apply_date", &launch_date as &(dyn ToSql + Sync))
        ]);
        q_insert(
            C_ADMINISTRATIVE_REFINES_HISTORY,
            &values,
            false,
        );
    }
}


//* func retrieves the last date before given cDate in which the value is refined
//old_name_was getAdmValue
#[allow(unused, dead_code)]
pub fn get_administrative_value(
    polling_key: &str,
    c_date: &CDateT) -> String
{
    let (_status, records) = q_select(
        C_ADMINISTRATIVE_REFINES_HISTORY,
        vec!["arh_value"],
        vec![
            simple_eq_clause("arh_subject", &polling_key.to_string()),
            ModelClause {
                m_field_name: "arh_apply_date",
                m_field_single_str_value: &c_date as &(dyn ToSql + Sync),
                m_clause_operand: "<=",
                m_field_multi_values: vec![],
            },
        ],
        vec![
            &OrderModifier { m_field: "arh_apply_date", m_order: "DESC" },
        ],
        1,
        true);

    if records.len() == 0
    {
        let err_msg = format!("Invalid arh_apply_date for ({}) on date({})", polling_key, c_date);
        dlog(
            &err_msg,
            constants::Modules::Sec,
            constants::SecLevel::Error);

        panic!("{}", err_msg);
    }

    return records[0]["arh_value"].clone();
}

//old_name_was getSingleIntegerValue
#[allow(unused, dead_code)]
pub fn get_single_integer_value(
    polling_key: &str,
    c_date: &CDateT) -> CMPAIValueT
{
    // fetch from DB the price for calculation Date
    let val = get_administrative_value(polling_key, c_date);
    dlog(
        &format!(
            "Retrieving Refine Request for ({}) on date({}) -> value({})",
            polling_key, c_date, val.to_string()),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    let val_f64 = val.parse::<f64>().unwrap();
    return val_f64 as CMPAIValueT;
}

//old_name_was getBasicTxDPCost
pub fn get_basic_transaction_data_and_process_cost(
    doc_length: DocLenT,
    c_date: &CDateT) -> CMPAIValueT
{
    let tx_base_price: CMPAIValueT = get_single_integer_value(polling_types::REQUEST_FOR_REFINE_BASE_PRICE, c_date);

    // * TODO: maybe can be modified in next version of transaction to be more fair
    // * specially after implementing the indented bach32 unlockers(recursively unlimited unlockers which have another bech32 as an unlocker)
    let (_x, _y, _gain, rev_gain) = cutils::calc_log(
        doc_length as i64,
        constants::MAX_DOC_LENGTH_BY_CHAR as i64,
        1);//(dLen * 1) / (iConsts.MAX_DOC_LENGTH_BY_CHAR * 1);
    println!("::::: tx_base_price {}", tx_base_price);
    println!("::::: rev_gain {}", rev_gain);
    // let powered = 10_000.0 * rev_gain;
    // println!("::::: powered {}", powered);
    let cost = cutils::i_floor_float(tx_base_price as f64 * rev_gain * 1000.0);
    println!("::::: cost {}", cost);
    return cost as CMPAIValueT;
}

//old_name_was getBasePricePerChar
#[allow(unused, dead_code)]
pub fn get_base_price_per_char(c_date: &CDateT) -> CMPAIValueT
{
    return get_single_integer_value(
        polling_types::REQUEST_FOR_REFINE_BASE_PRICE,
        c_date);
}

//old_name_was getPollingDPCost
#[allow(unused, dead_code)]
pub fn get_polling_data_and_process_cost(c_date: &CDateT) -> CMPAIValueT
{
    return get_single_integer_value(polling_types::REQUEST_FOR_REFINE_POLLING_PRICE, c_date);
}

//old_name_was getPledgeDPCost
#[allow(unused, dead_code)]
pub fn get_pledge_data_and_process_cost(c_date: &CDateT) -> CMPAIValueT
{
    return get_single_integer_value(polling_types::REQUEST_FOR_REFINE_PLEDGE_PRICE, c_date);
}

//old_name_was getClosePledgeDPCost
#[allow(unused, dead_code)]
pub fn get_close_pledge_data_and_process_cost(c_date: &CDateT) -> CMPAIValueT
{
    return get_single_integer_value(polling_types::REQUEST_FOR_REFINE_CONCLUDE_PLEDGE_PRICE, c_date);
}

//old_name_was getCloseDNAProposalDPCost
#[allow(unused, dead_code)]
pub fn get_close_proposal_data_and_process_cost(c_date: &CDateT) -> CMPAIValueT
{
    return get_single_integer_value(polling_types::REQUEST_FOR_REFINE_PROPSAL_PRICE, c_date);
}

//old_name_was getBallotDPCost
#[allow(unused, dead_code)]
pub fn get_ballot_data_and_process_cost(c_date: &CDateT) -> CMPAIValueT
{
    return get_single_integer_value(polling_types::REQUEST_FOR_REFINE_BALLOT_PRICE, c_date);
}

//old_name_was getINameRegDPCost
#[allow(unused, dead_code)]
pub fn get_i_name_reg_data_and_process_cost(c_date: &CDateT) -> CMPAIValueT
{
    return get_single_integer_value(polling_types::REQUEST_FOR_REFINE_INAME_REGISTER_PRICE, c_date);
}

//old_name_was getINameBindDPCost
#[allow(unused, dead_code)]
pub fn get_i_name_bind_data_and_process_cost(c_date: &CDateT) -> CMPAIValueT
{
    return get_single_integer_value(polling_types::REQUEST_FOR_REFINE_INAME_BIND_PGP_PRICE, c_date);
}

//old_name_was getINameMsgDPCost
#[allow(unused, dead_code)]
pub fn get_i_name_msg_data_and_process_cost(c_date: &CDateT) -> CMPAIValueT
{
    return get_single_integer_value(polling_types::REQUEST_FOR_REFINE_INAME_MESSAGE_PRICE, c_date);
}

//old_name_was getCPostDPCost
#[allow(unused, dead_code)]
pub fn get_free_post_data_and_process_cost(c_date: &CDateT) -> CMPAIValueT
{
    return get_single_integer_value(polling_types::REQUEST_FOR_REFINE_FREE_POST_PRICE, c_date);
}

//old_name_was getBlockFixCost
#[allow(unused, dead_code)]
pub fn get_block_fix_cost(c_date: &CDateT) -> CMPAIValueT
{
    return get_single_integer_value(polling_types::REQUEST_FOR_REFINE_BLOCK_FIXED_COST, c_date);
}


// TODO: optimize it to catch price-set and fetch it once in every 12 hours
//old_name_was prepareDocExpenseDict
pub fn prepare_doc_expense_dict(
    c_date: &CDateT,
    doc_len: DocLenT) -> HashMap<&str, CMPAIValueT>
{
    let service_prices: HashMap<&str, CMPAIValueT> = HashMap::from([
        (constants::document_types::BASIC_TX, get_basic_transaction_data_and_process_cost(doc_len, c_date)),
        (constants::document_types::ADMINISTRATIVE_POLLING, get_polling_data_and_process_cost(c_date)),
        (constants::document_types::POLLING, get_polling_data_and_process_cost(c_date)),
        (constants::document_types::PLEDGE, get_pledge_data_and_process_cost(c_date)),
        (constants::document_types::CLOSE_PLEDGE, get_close_pledge_data_and_process_cost(c_date)),
        (constants::document_types::PROPOSAL, get_close_proposal_data_and_process_cost(c_date)),
        (constants::document_types::BALLOT, get_ballot_data_and_process_cost(c_date)),
        (constants::document_types::I_NAME_REGISTER, get_i_name_reg_data_and_process_cost(c_date)),
        (constants::document_types::I_NAME_BIND, get_i_name_bind_data_and_process_cost(c_date)),
        (constants::document_types::I_NAME_MESSAGE_TO, get_i_name_msg_data_and_process_cost(c_date)),
        (constants::document_types::FREE_POST, get_free_post_data_and_process_cost(c_date))
    ]);

    return service_prices;
}

//old_name_was getDocExpense
pub fn get_doc_expense(
    doc_type: &str,
    doc_len: DocLenT,
    _doc_class: &str,
    c_date: &CDateT) -> CMPAIValueT
{
    if doc_len > constants::MAX_DOC_LENGTH_BY_CHAR
    {
        dlog(
            &format!("Doc length is bigger then permission {}>{}", doc_len, constants::MAX_DOC_LENGTH_BY_CHAR),
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return 0;
    }

    let service_prices = prepare_doc_expense_dict(c_date, doc_len);

    if service_prices.keys().cloned().collect::<Vec<&str>>().contains(&doc_type)
    {
        return service_prices[doc_type];
    }

    dlog(
        &format!("Unknown doc type to extract expense: {}", doc_type),
        constants::Modules::Sec,
        constants::SecLevel::Warning);

    return 0;

    //TODO: implement plugin price
    // if type of documents is not defined, so accept it as a base feePerByte
//  let pluginPrice = listener.doCallSync('SASH_calc_service_price', args);
//  if (_.has(pluginPrice, 'err')&& pluginPrice.err != false) {
//      utils.exiter("wrong plugin price calc for ${utils.stringify(args)}", 434);
//  }
//  if (!_.has(pluginPrice, 'fee')) {
//    utils.exiter("missed price fee for doc_type(${doc_type})", 34)
//  }
//  return pluginPrice.fee;
}

//old_name_was getTransactionMinimumFee
pub fn get_transaction_minimum_fee(c_date: &CDateT) -> CMPAIValueT
{
    return
        constants::TRANSACTION_MINIMUM_LENGTH as CMPAIValueT *
            get_base_price_per_char(c_date) *
            get_doc_expense(
                constants::document_types::BASIC_TX,
                constants::TRANSACTION_MINIMUM_LENGTH,
                "",
                c_date);
}

/*

uint8_t SocietyRules::getPoWDifficulty(cDate: &CDateT)
{
  if (cDate < "2020-10-01 00:00:00") {
    return 4;
  } else if (("2020-10-01 00:00:00" <= cDate) &&  (cDate < "2021-00-01 00:00:00")) {
    return 6;
  } else if (("2021-00-01 00:00:00" <= cDate) && (cDate < "2021-06-01 00:00:00")) {
    return 10;
  } else {
    return 18;
  }
}

*/

//old_name_was getMaxBlockSize
#[allow(unused, dead_code)]
pub fn get_max_block_size(block_type: &String) -> BlockLenT
{
    // TODO: implement it to retrieve max number from db, by voting process

    let default_max: BlockLenT = 10_000_000 * 10; //MAX_BLOCK_LENGTH_BY_CHAR
    let max_block_size: HashMap<String, BlockLenT> = HashMap::from([
        (constants::block_types::NORMAL.to_string(), default_max),
        (constants::block_types::COINBASE.to_string(), default_max),
        (constants::block_types::REPAYMENT_BLOCK.to_string(), default_max),
        (constants::block_types::FLOATING_SIGNATURE.to_string(), default_max),
        (constants::block_types::SUS_BLOCK.to_string(), default_max),
        (constants::block_types::FLOATING_VOTE.to_string(), default_max),
        (constants::block_types::POW.to_string(), 7100 as BlockLenT)]);

    if max_block_size.keys().cloned().collect::<Vec<String>>().contains(block_type)
    {
        return max_block_size[block_type].clone();
    }

    dlog(
        &format!("Invalid block type! for length block_type({})", block_type),
        constants::Modules::Sec,
        constants::SecLevel::Error);

    return 0 as BlockLenT;
}

//old_name_was getSingleFloatValue
pub fn get_single_float_value(
    polling_key: &String,
    c_date: &CDateT) -> f64
{
    if !cutils::is_a_valid_date_format(c_date)
    {
        panic!(
            "Invalid c-date for get Single Float Value for pollingKey({}) cDate:({}) ",
            polling_key,
            c_date
        );
    }
    // fetch from DB the price for calculation Date
    let value_ = get_administrative_value(
        polling_key,
        c_date);

    let value_ = cutils::i_floor_float(value_.parse::<f64>().unwrap());
    dlog(
        &format!("RFRf(float) for pollingKey({}) cDate:({}) => value({})",
                 polling_key,
                 c_date,
                 value_
        ),
        constants::Modules::Sec,
        constants::SecLevel::Error);

    return value_;
}

//  -  -  -  shares parameters settings
//old_name_was getMinShareToAllowedIssueFVote
pub fn get_min_share_to_allowed_issue_f_vote(c_date: &CDateT) -> SharesPercentT
{
    return get_single_float_value(
        &polling_types::REQUEST_FOR_REFINE_MIN_SHARES_TO_FLOATING_VOTES.to_string(),
        c_date);
}

/*
SharesPercentT SocietyRules::getMinShareToAllowedVoting(cDate: &CDateT)
{
  return get_single_float_value(polling_types::RFRfMinS2V, cDate);
}

SharesPercentT SocietyRules::getMinShareToAllowedSignCoinbase(cDate: &CDateT)
{
  return get_single_float_value(polling_types::RFRfMinFSign, cDate);
}

SharesPercentT SocietyRules::getMinShareToAllowedWiki(cDate: &CDateT)
{
  return get_single_float_value(polling_types::RFRfMinS2Wk, cDate);
}

SharesPercentT SocietyRules::getMinShareToAllowedDemos(cDate: &CDateT)
{
  return get_single_float_value(polling_types::RFRfMinS2DA, cDate);
}

bool SocietyRules::logRefineDetail(
  const CDocHashT& arh_hash,
  const String& arh_subject,
  const double arh_value,
  const String& arh_apply_date)
{
  /**
  * FIXME: since "arh_apply_date" is calculted based on container block.creation Date
  * it is possible for a same pSubject having 2 or more different polling which the arh_apply_date
  * will be sam(e.g they reside in same block or 2 different block by same creation date which is heigly possible
  * an adversor create them)
  * and at the end we have different result with same date to apply!
  * it must be fixed ASAP, meanwhile the comunity can simply futile the duplicated polling by negative votes
  */

  QueryRes exist = DbModel::select(
    stbl_administrative_refines_history,
    {"arh_hash"},
    {{"arh_hash", arh_hash}});
  if (exist.records.len() > 0)
  {
    CLog::log("duplicate refine hist (" + cutils::hash8c(arh_hash) + ")", "app", "error");
    return false;
  }
  DbModel::insert(
    stbl_administrative_refines_history,
    {{"arh_hash", arh_hash},
    {"arh_subject", arh_subject},
    {"arh_value", arh_value},
    {"arh_apply_date", arh_apply_date}});

  return true;
}

bool SocietyRules::treatPollingWon(
  const QVDicT& polling,
  const CDateT& approveDate)
{
  CDocHashT admPollingHash = polling["pll_ref"].to_string();
  QueryRes admPollings = DbModel::select(
    stbl_administrative_pollings,
    stbl_administrative_pollings_fields,
    {{"apr_hash", admPollingHash}});
  if (admPollings.records.len() != 1)
  {
    CLog::log("Invalid winner adm olling! admPolling(" + cutils::hash8c(admPollingHash) + ")", "app", "error");
    return false;
  }
  QVDicT admPolling = admPollings.records[0];
  JSonObject the_values = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(admPolling["apr_values"].to_string()).content);
  CLog::log("Treat Polling Won adm Polling: " + cutils::dumpIt(admPolling), "app", "trace");
  CDateT arh_apply_date = cutils::getACycleRange(
    approveDate,
    0,
    2).from;

  if (VString {"RFRfMinS2V", "RFRfMinFSign", "RFRfMinFVote"}.contains(admPolling["apr_subject"].to_string()))
  {
    logRefineDetail(
      admPollingHash,
      admPolling["apr_subject"].to_string(),
      the_values["pShare"].toDouble(), //.share,
      arh_apply_date);

  } else if (VString {
    "RFRfBasePrice",
    "RFRfTxBPrice",
    "RFRfBlockFixCost",
    "RFRfPollingPrice",
    "RFRfPLedgePrice",
    "RFRfClPLedgePrice",
    "RFRfDNAPropPrice",
    "RFRfBallotPrice",
    "RFRfINameRegPrice",
    "RFRfINameBndPGPPrice",
    "RFRfINameMsgPrice",
    "RFRfFPostPrice"}.contains(admPolling["apr_subject"].to_string()))
  {
    logRefineDetail(
      admPollingHash,
      admPolling["apr_subject"].to_string(),
      the_values["pFee"].toDouble(),  //.pFee,
      arh_apply_date);

  } else {
    CLog::log("Unknown apr_subject in 'treat Polling Won' " + admPolling["apr_subject"].to_string(), "sec", "error");
    return false;
  }

  // update proposal
  DbModel::update(
    stbl_administrative_pollings,
    {{"apr_conclude_date", approveDate},
    {"apr_approved", constants::YES}},
    {{"apr_hash", admPollingHash}});

  return true;
}


bool SocietyRules::concludeAdmPolling(
  const QVDicT& polling,
  const CDateT& approveDate)
{
  CDocHashT admPollingHash = polling["pll_ref"].to_string();

  // update proposal
  DbModel::update(
    stbl_administrative_pollings,
    {{"apr_conclude_date", approveDate},
    {"apr_approved", constants::NO}},
    {{"apr_hash", admPollingHash}});

  return true;
}


bool SocietyRules::recordAnAdministrativePollingInDB(
  const Block& block,
  const AdministrativePollingDocument* polling)
{
  CLog::log("record A Society administrative polling document: " + cutils::hash8c(polling->get_doc_hash()), "app", "trace");


  QueryRes dbl = DbModel::select(
    stbl_administrative_pollings,
    {"apr_hash"},
    {{"apr_hash", polling.m_doc_hash}});
  if (dbl.records.len() > 0)
  {
    CLog::log("Try to double insert existed adm polling(" + cutils::hash8c(polling.m_doc_hash) + ")", "sec", "error");
    return true;  // by the way, it is ok and the block must be recorded in DAG
  }

  QVDicT values {
    {"apr_hash", polling->get_doc_hash()},
    {"apr_creator", polling.m_polling_creator},
    {"apr_subject", polling.m_polling_subject},
    {"apr_values", BlockUtils::wrapSafeContentForDB(cutils::serializeJson(polling.m_polling_values)).content},
    {"apr_comment", polling.m_doc_comment},
    {"apr_creation_date", block.m_block_creation_date},
    {"apr_conclude_date", ""},
    {"apr_approved", constants::NO},
    {"apr_conclude_info", BlockUtils::wrapSafeContentForDB(cutils::getANullStringifyedJsonObj()).content}
  };
  CLog::log("New admPolling is creates: " + cutils::dumpIt(values), "app", "trace");
  bool res = DbModel::insert(
    stbl_administrative_pollings,
    values);
  return res;
}

bool SocietyRules::removeAdmPolling(const CDocHashT& doc_hash)
{
  DbModel::dDelete(
    stbl_administrative_pollings,
    {{"apr_hash", doc_hash}});

  return true;
}

QVDRecordsT SocietyRules::searchInAdmPollings(
  const ClausesT& clauses,
  const VString& fields,
  const OrderT& order,
  const int limit)
{
  QueryRes posts = DbModel::select(
    stbl_administrative_pollings,
    fields,
    clauses,
    order,
    limit);

  return posts.records;
}

QVDRecordsT SocietyRules::searchInAdmRefineHistory(
  const ClausesT& clauses,
  const VString& fields,
  const OrderT& order,
  const int limit)
{
  QueryRes posts = DbModel::select(
    stbl_administrative_refines_history,
    fields,
    clauses,
    order,
    limit);

  return posts.records;
}

QVDicT SocietyRules::readAdministrativeCurrentValues()
{
  /**
  * instead of fetching from a single table, tha values are retrived from proper functions in ConfParamsHandler
  */
  CDateT cDate = application().now();
  CDateT cycleStartDate = cutils::getACycleRange().from;

  QVDicT res {
    {"cycleStartDate", cycleStartDate},
    {"transactionMinimumFee", QVariant::fromValue(get_transaction_minimum_fee(cDate))},
    {"docExpenseDict", prepareDocExpenseDict(cDate, constants::TRANSACTION_MINIMUM_LENGTH)},
    {"basePricePerChar", QVariant::fromValue(getBasePricePerChar(cDate))},
    {"blockFixCost", QVariant::fromValue(get_block_fix_cost(cDate))},
    {"minShareToAllowedIssueFVote", get_min_share_to_allowed_issue_f_vote(cDate)},
    {"minShareToAllowedVoting", getMinShareToAllowedVoting(cDate)},
    {"minShareToAllowedSignCoinbase", getMinShareToAllowedSignCoinbase(cDate)}
  };

  return res;
}

JSonArray SocietyRules::loadAdmPollings(
  cDate: &CDateT)
{
  JSonArray admPollings = {
    JSonObject {
      {"key", polling_types::RFRfBasePrice},
      {"label", "Request for Refine charachter base pice of Data & Process costs(DPCost), currently is " + cutils::microPAIToPAI6(getBasePricePerChar(cDate)) + " PAI per Char"},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(getBasePricePerChar(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}
      }}
    },

    JSonObject {
      {"key", polling_types::RFRfTxBPrice},
      {"label", "Request for Refine Transaction DPCost, currently is " + cutils::microPAIToPAI6(cutils::CFloor(getBasicTxDPCost(constants::TRANSACTION_MINIMUM_LENGTH, cDate))) + " micro PAI per Char"},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(getBasicTxDPCost(constants::TRANSACTION_MINIMUM_LENGTH, cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },

    JSonObject {
      {"key", polling_types::RFRfBlockFixCost},
      {"label", "Request for Refine Block Fix Cost, currently is " + cutils::microPAIToPAI6(get_block_fix_cost(cDate)) + " micro PAI per Block"},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(get_block_fix_cost(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfPollingPrice},
      {"label", "Request for Refine DPCost of Polling Document, currently is " + cutils::microPAIToPAI6(getPollingDPCost(cDate)) + " PAIs per char"},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(getPollingDPCost(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfPLedgePrice},
      {"label", "Request for Refine DPCost of Pledge Document, currently is " + cutils::microPAIToPAI6(getPledgeDPCost(cDate)) + " PAIs per char"},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(getPledgeDPCost(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfClPLedgePrice},
      {"label", "Request for Refine DPCost of Close a Pledged Account, currently is " + cutils::microPAIToPAI6(getClosePledgeDPCost(cDate)) + " PAIs per char"},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(getClosePledgeDPCost(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfDNAPropPrice},
      {"label", "Request for Refine DPCost of offer a DNAProposal, currently is " + cutils::microPAIToPAI6(getCloseDNAProposalDPCost(cDate)) + " PAIs per char"},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(getCloseDNAProposalDPCost(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfBallotPrice},
      {"label", "Request for Refine DPCost of Ballot, currently is " + cutils::microPAIToPAI6(getBallotDPCost(cDate)) + " PAIs per char"},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(getBallotDPCost(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfINameRegPrice},
      {"label", "Request for Refine DPCost of register an iName, currently is " + cutils::microPAIToPAI6(getINameRegDPCost(cDate)) + " PAIs per unit "},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(getINameRegDPCost(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfINameBndPGPPrice},
      {"label", "Request for Refine DPCost of Binding an iPGP key to an iName, currently is " + cutils::microPAIToPAI6(getINameBindDPCost(cDate)) + " PAIs per a pair-key "},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(getINameBindDPCost(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfINameMsgPrice},
      {"label", "Request for Refine DPCost of a message via DAG, currently is " + cutils::microPAIToPAI6(getINameMsgDPCost(cDate)) + " PAIs per char. it refers to entire encrypted message and head & tail & ..."},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(getINameMsgDPCost(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfFPostPrice},
      {"label", "Request for Refine DPCost of a Free Post (including text, file, media...), currently is " + cutils::microPAIToPAI6(getCPostDPCost(cDate)) + " PAIs per char"},
      {"pValues", JSonObject {
        {"pFee", QVariant::fromValue(getCPostDPCost(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfMinS2Wk},
      {"label", "Request for Refine Minimum Shares to be Allowed to participate in Wiki Activities, currently is " + String::number(getMinShareToAllowedWiki(cDate)) + " Percent"},
      {"pValues", JSonObject {
        {"pShare", QVariant::fromValue(getMinShareToAllowedWiki(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfMinS2DA},
      {"label", "Request for Refine Minimum Shares to be Allowed to participate in Demos Discussions, currently is " + String::number(getMinShareToAllowedDemos(cDate)) + " Percent"},
      {"pValues", JSonObject {
        {"pShare", QVariant::fromValue(getMinShareToAllowedDemos(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfMinS2V},
      {"label", "Request for Refine Minimum Shares to be Allowed to participate in ellections, currently is " + String::number(getMinShareToAllowedVoting(cDate)) + " Percent"},
      {"pValues", JSonObject {
        {"pShare", QVariant::fromValue(getMinShareToAllowedVoting(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfMinFSign},
      {"label", "Request for Refine Minimum Shares to be Allowed to Sign a Coinbase block, currently is " + String::number(getMinShareToAllowedSignCoinbase(cDate)) + " Percent"},
      {"pValues", JSonObject {
        {"pShare", QVariant::fromValue(getMinShareToAllowedSignCoinbase(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRfMinFVote},
      {"label", "Request for Refine Minimum Shares to be Allowed to Issue a Floating Vote (either a block or an entry), currently is " + String::number(get_min_share_to_allowed_issue_f_vote(cDate)) + " Percent"},
      {"pValues", JSonObject {
        {"pShare", QVariant::fromValue(get_min_share_to_allowed_issue_f_vote(cDate)).toDouble()},
        {"pTimeframe", QVariant::fromValue(CMachine::getMinPollingTimeframeByHour()).toDouble()}}
      }
    },
    JSonObject {
      {"key", polling_types::RFRlRsCoins},
      {"label", "Request for release a Reserved Block"},
    },
  };

  return admPollings;
}

std::tuple<bool, String> SocietyRules::createAPollingFor(
  const String& polling_subject,
  TimeByHoursT voting_timeframe,
  double the_value) // it can be fee or shares
{
  auto[total_shares_, share_amount_per_holder, holdersOrderByShares_] = DNAHandler::getSharesInfo();
  uint64_t voters_count = share_amount_per_holder.keys().len();
  voting_timeframe = PollingHandler::normalizeVotingTimeframe(voting_timeframe);
  auto[status, msg] = PollingHandler::makeReqForAdmPolling(
    polling_subject,
    voting_timeframe,
    the_value,
    voters_count);
  return {status, msg};
}

HashMap<uint32_t, QVDicT> SocietyRules::getOnchainSocietyPollings(
  const CAddressT& voter)
{
  // retrieve machine votes
  QVDRecordsT votes = BallotHandler::searchInLocalBallots();
  QV2DicT local_votes_dict {};
  for (QVDicT a_vote: votes)
    local_votes_dict[a_vote["lbt_pll_hash"].to_string()] = a_vote;

  String complete_query = R"(
    SELECT ppr.ppr_name, ppr.ppr_perform_type, ppr.ppr_votes_counting_method,

    apr.apr_hash, apr.apr_creator, apr.apr_subject, apr.apr_values, apr.apr_comment, apr.apr_creation_date,
    apr.apr_conclude_date, apr.apr_approved, apr.apr_conclude_info,

    pll.pll_hash, pll.pll_ref, pll.pll_start_date, pll.pll_end_date, pll.pll_timeframe, pll.pll_status, pll.pll_ct_done,
    pll.pll_y_count, pll.pll_n_count, pll.pll_a_count,
    pll.pll_y_shares, pll.pll_n_shares, pll.pll_a_shares,
    pll.pll_y_gain, pll.pll_n_gain, pll.pll_a_gain,
    pll.pll_y_value, pll.pll_n_value, pll.pll_a_value

    FROM c_pollings pll
    JOIN c_polling_profiles ppr ON ppr.ppr_name=pll.pll_class
    JOIN c_administrative_pollings apr ON apr.apr_hash = pll.pll_ref
  )";

  if (constants::DATABASAE_AGENT == "psql")
  {
    complete_query += "WHERE pll.pll_ref_type='AdmPolling' ORDER BY pll.pll_start_date ";
  }
  else if (constants::DATABASAE_AGENT == "sqlite")
  {
    complete_query += "WHERE pll.pll_ref_type=\"AdmPolling\" ORDER BY pll.pll_start_date ";
  }

  QueryRes res = DbModel::customQuery(
    "db_comen_general",
    complete_query,
    {"ppr_name", "ppr_perform_type", "ppr_votes_counting_method",
      "apr_hash", "apr_creator", "apr_subject", "apr_values", "apr_comment",
      "apr_conclude_date", "apr_approved", "apr_conclude_info",
      "pll_hash", "pll_ref", "pll_start_date", "pll_end_date", "pll_timeframe", "pll_status", "pll_ct_done",
      "pll_y_count", "pll_n_count", "pll_a_count",
      "pll_y_shares", "pll_n_shares", "pll_a_shares",
      "pll_y_gain", "pll_n_gain", "pll_a_gain",
      "pll_y_value", "pll_n_value", "pll_a_value"},
    0,
    {},
    false,
    false);

  HashMap<uint32_t, QVDicT> final_result {};
  uint32_t result_number = 0;
  for (QVDicT a_society_polling: res.records)
  {
    uint32_t row_inx = result_number * 10; // 10 rows for each proposal are needed

    // calc potentiasl voter gains
    if (voter != "")
    {
      uint64_t diff = application().time_diff(a_society_polling["pll_start_date"].to_string()).asMinutes;
      auto[yes_gain, no_abstain_gain, latenancy_] = PollingHandler::calculateVoteGain(
        diff,
        diff,
        a_society_polling["pll_timeframe"].toDouble() * 60.0);
      Q_UNUSED(latenancy_);

//      let vGain = pollHandler.calculateVoteGain(diff, diff, a_society_polling.pll_timeframe * 60);
      a_society_polling["your_yes_gain"] = cutils::customFloorFloat(yes_gain * 100, 2);
      a_society_polling["your_abstain_gain"] = cutils::customFloorFloat(no_abstain_gain * 100, 2);
      a_society_polling["your_no_gain"] = cutils::customFloorFloat(no_abstain_gain * 100, 2);

    } else {
      a_society_polling["your_yes_gain"] = 0.0;
      a_society_polling["your_abstain_gain"] = 0.0;
      a_society_polling["your_no_gain"] = 0.0;
    }

    CDateT conclude_date = a_society_polling["pr_conclude_date"].to_string();
    if (conclude_date == "")
    {
      CDateT polling_end_date = a_society_polling["pll_end_date"].to_string();
      CDateT approve_date_ = cutils::minutesAfter(cutils::get_cycle_by_minutes() * 2, polling_end_date);
      conclude_date = cutils::get_coinbase_range(approve_date_).from;
    }

    String win_complementary_text = "";
    String win_complementary_tip = "";
    if (a_society_polling["pll_ct_done"].to_string() == constants::YES)
    {
      QVDRecordsT dna_records = DNAHandler::searchInDNA(
        {{"dn_doc_hash", a_society_polling["pll_ref"].to_string()}});
      if (dna_records.len() > 0)
      {
        win_complementary_text = " Shares created on " + dna_records[0]["dn_creation_date"].to_string();
        win_complementary_tip = "First income on " + cutils::minutesAfter(
          constants::SHARE_MATURITY_CYCLE * cutils::get_cycle_by_minutes(),
          dna_records[0]["dn_creation_date"].to_string());
      }
    } else{
      win_complementary_tip = "First income (if win) on " + cutils::minutesAfter(
        constants::SHARE_MATURITY_CYCLE * cutils::get_cycle_by_minutes(), conclude_date);
    }

    final_result[row_inx] = QVDicT {
      {"apr_hash", a_society_polling["apr_hash"]},
      {"apr_comment", a_society_polling["apr_comment"]},
      {"polling_number", result_number + 1},
      {"pr_contributor_account", a_society_polling["pr_contributor_account"]},
      {"pll_status", constants::STATUS_TO_LABEL[a_society_polling["pll_status"].to_string()]},
      {"ppr_name", a_society_polling["ppr_name"]},
    };


    final_result[row_inx + 1] = QVDicT {
      {"apr_hash", a_society_polling["apr_hash"]},
      {"apr_creator", a_society_polling["apr_creator"]},
      {"pll_ct_done", constants::STATUS_TO_LABEL[a_society_polling["pll_ct_done"].to_string()]},
      {"the_conclude_date", conclude_date},
      {"ppr_perform_type", a_society_polling["ppr_perform_type"]},
    };

    final_result[row_inx + 2] = QVDicT {
      {"apr_hash", a_society_polling["apr_hash"]},
      {"pll_hash", a_society_polling["pll_hash"]},
      {"ppr_votes_counting_method", a_society_polling["ppr_votes_counting_method"]},
    };

    final_result[row_inx + 3] = QVDicT {
      {"apr_hash", a_society_polling["apr_hash"]},
      {"pr_descriptions", a_society_polling["pr_descriptions"]},
      {"", a_society_polling["")},
 ]  };

    CDateT end_y_date;
    if (application().cycle() == 1)
    {
      end_y_date = cutils::minutesAfter(a_society_polling["pll_timeframe"].toDouble() * 60, a_society_polling["pll_start_date"].to_string());

    } else{
      // test ambient
      TimeByHoursT yes_timeframe_by_minutes = static_cast<uint64_t>(a_society_polling["pll_timeframe"].toDouble() * 60.0);
      CLog::log("yes_timeframe_by_minutes____pll_timeframe" + String::number(a_society_polling["pll_timeframe"].toDouble()));
      CLog::log("yes_timeframe_by_minutes____" + String::number(yes_timeframe_by_minutes));
      end_y_date = cutils::minutesAfter(yes_timeframe_by_minutes, a_society_polling["pll_start_date"].to_string());

    }


    final_result[row_inx + 4] = QVDicT {
      {"apr_hash", a_society_polling["apr_hash"]},
      {"pr_tags", a_society_polling["pr_tags"]},
      {"pll_start_date", a_society_polling["pll_start_date"]},
      {"pll_y_count", a_society_polling["pll_y_count"]},
      {"pll_y_shares", a_society_polling["pll_y_shares"]},
      {"pll_y_gain", a_society_polling["pll_y_gain"]},
      {"pll_y_value", a_society_polling["pll_y_value"]},
      {"your_yes_gain", a_society_polling["your_yes_gain"]},
    };

    final_result[row_inx + 5] = QVDicT {
      {"apr_hash", a_society_polling["apr_hash"]},
      {"pr_help_hours", a_society_polling["pr_help_hours"]},
      {"end_y", end_y_date}, //cutils::minutesAfter(yes_timeframe * 60, a_society_polling["pll_start_date"].to_string())},
      {"pll_a_count", a_society_polling["pll_a_count"]},
      {"pll_a_shares", a_society_polling["pll_a_shares"]},
      {"pll_a_gain", a_society_polling["pll_a_gain"]},
      {"pll_a_value", a_society_polling["pll_a_value"]},
      {"your_abstain_gain", a_society_polling["your_abstain_gain"]},
    };

    final_result[row_inx + 6] = QVDicT {
      {"apr_hash", a_society_polling["apr_hash"]},
      {"pr_help_level", a_society_polling["pr_help_level"]},
      {"end_n", a_society_polling["pll_end_date"]},
      {"pll_n_count", a_society_polling["pll_n_count"]},
      {"pll_n_shares", a_society_polling["pll_n_shares"]},
      {"pll_n_gain", a_society_polling["pll_n_gain"]},
      {"pll_n_value", a_society_polling["pll_n_value"]},
      {"your_no_gain", a_society_polling["your_no_gain"]},
    };

    final_result[row_inx + 7] = QVDicT {
      {"apr_hash", a_society_polling["apr_hash"]},
      {"proposed_shares", a_society_polling["pr_help_level"].toInt() * a_society_polling["pr_help_hours"].toInt()},
      {"", a_society_polling["")},
 ]    {"", a_society_polling["")},
 ]  };

    String final_status_color, final_status_text;
    if (a_society_polling["pll_y_value"].toUInt() >= a_society_polling["pll_n_value"].toUInt())
    {
      final_status_color = "00ff00";
      final_status_text = "Approved (";

    } else {
      final_status_color = "ff0000";
      final_status_text = "Missed (";

    }
    final_status_text += cutils::sep_num_3(a_society_polling["pll_y_value"].toUInt() - a_society_polling["pll_n_value"].toUInt()) +" points)";
    final_status_text += win_complementary_text;

    final_result[row_inx + 8] = QVDicT {
      {"apr_hash", a_society_polling["apr_hash"]},
      {"final_status_color", final_status_color},
      {"final_status_text", final_status_text},
      {"win_complementary_tip", win_complementary_tip},
    };

    final_result[row_inx + 9] = QVDicT {
      {"apr_hash", a_society_polling["apr_hash"]},
    };

//    a_society_polling.pllEndDateAbstainOrNo = utils.minutesAfter(utils.floor(a_society_polling.pll_timeframe * 60 * 1.5), a_society_polling.pll_start_date);
//    a_society_polling.machineBallot = _.has(local_votes_dict, a_society_polling.pll_hash) ? local_votes_dict[a_society_polling.pll_hash] : null;


    result_number++;
  }

  return final_result;
}

 */