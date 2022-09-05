use std::collections::HashMap;
use postgres::types::ToSql;
use crate::lib::custom_types::{CAddressT, CBlockHashT, CDateT, CDocHashT, CMPAISValueT, COutputIndexT, QVDRecordsT, VString};
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, q_delete, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::{C_MACHINE_WALLET_FUNDS, C_MACHINE_WALLET_FUNDS_FIELDS};
use crate::{application, constants, dlog, get_value, machine};
use crate::lib::block::document_types::basic_tx_document::BasicTxDocument;
use crate::lib::dag::dag::search_in_dag;
use crate::lib::k_v_handler::upsert_kvalue;
use crate::lib::wallet::update_funds_from_new_block::update_funds_from_new_block;
use crate::lib::wallet::wallet_address_handler::{get_addresses_list};

/*

#include "stable.h"

#include "lib/block_utils.h"
#include "lib/block/document_types/document.h"
#include "lib/services/society_rules/society_rules.h"
#include "lib/block/document_types/basic_tx_document.h"
#include "lib/dag/normal_block/rejected_transactions_handler.h"

#include "wallet.h"


// js name was retrieveSpendableUTXOsAsync
QVDRecordsT Wallet::retrieveSpendableCoins(StringList w_addresses)
{
  StringList wallet_ddresses {};

  if (w_addresses.len() == 0)
  {
    auto[address_records, details] = getAddressesList();
    Q_UNUSED(details);
    for(QVDicT add: address_records)
      wallet_ddresses.push(add["wa_address"].to_string());
  }
  QVDRecordsT UTXOs = extract_coins_by_addresses(wallet_ddresses);
  return UTXOs;
}

*/
//old_name_was refreshCoins
pub fn refresh_coins() -> bool
{
    let mp_code = machine().get_selected_m_profile();

    //prepare the wallet addreses:
    let (addresses_, _details) =
        get_addresses_list(&mp_code, vec!["wa_address"], false);
    let mut addresses: VString = vec![];
    for elm in &addresses_
    {
        addresses.push(elm["wa_address"].to_string());
    }

    if addresses.len() == 0
    {
        return false;
    }

    let latest_update = get_value("latest_refresh_funds");
    println!("xxxxxx latest refresh funds: {}", latest_update);
    dlog(
        &format!("latest refresh funds: {}", latest_update),
        constants::Modules::App,
        constants::SecLevel::Info);

    if latest_update == ""
    {
        let date_ = machine().get_launch_date();
        upsert_kvalue("latest_refresh_funds", &date_, false);
    }

    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "b_type",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "NOT IN",
        m_field_multi_values: vec![],
    };
    for a_type in &[constants::block_types::FLOATING_SIGNATURE, constants::block_types::FLOATING_VOTE, constants::block_types::POW]
    {
        c1.m_field_multi_values.push(a_type as &(dyn ToSql + Sync));
    }
    let launch_date = machine().get_launch_date();
    let block_records = search_in_dag(
        vec![
            c1,
            ModelClause {
                m_field_name: "b_creation_date",
                m_field_single_str_value: &launch_date as &(dyn ToSql + Sync),
                m_clause_operand: ">=",
                m_field_multi_values: vec![],
            },
        ],// TODO improve it to reduce process load. (e.g. use latest_update instead)
        vec!["b_type", "b_hash", "b_body"],
        vec![&OrderModifier { m_field: "b_creation_date", m_order: "ASC" }],
        0,
        false);

    // FIXME: (improve it) remove this and search in c_blocks only new blocks
    q_delete(
        C_MACHINE_WALLET_FUNDS,
        vec![simple_eq_clause("wf_mp_code", &mp_code)],
        false);

    println!("xxxxxx Going to update funds of {} blocks", block_records.len());
    dlog(
        &format!("Going to update funds of {} blocks", block_records.len()),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    dlog(
        &format!("Going to update funds of addresses: {:?} ", addresses),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    for a_block_records in &block_records
    {
        update_funds_from_new_block(a_block_records, &addresses);
    }

    let now_ = application().now();
    upsert_kvalue("latest_refresh_funds", &now_, false);

    return true;
}

//old_name_was getCoinsList
pub fn get_coins_list(should_refresh_coins: bool) -> QVDRecordsT
{
    if should_refresh_coins
    { refresh_coins(); }

    let mp_code = machine().get_selected_m_profile();

    let (_status, records) = q_select(
        C_MACHINE_WALLET_FUNDS,
        Vec::from(C_MACHINE_WALLET_FUNDS_FIELDS),
        vec![simple_eq_clause("wf_mp_code", &mp_code)],
        vec![
            &OrderModifier { m_field: "wf_mature_date", m_order: "ASC" }],
        0,
        false);

    return records;
}

//old_name_was insertAnUTXOInWallet
pub fn insert_a_coin_in_wallet(
    wf_block_hash: &CBlockHashT,
    wf_trx_hash: &CDocHashT,
    wf_o_index: COutputIndexT,
    wf_address: &CAddressT,
    wf_o_value: CMPAISValueT,
    wf_trx_type: &String,
    wf_creation_date: &CDateT,
    wf_mature_date: &CDateT,
    wf_mp_code: &String) -> bool
{
    println!("xxxxxx insert_a_coin_in_wallet");
    let mut wf_mp_code = wf_mp_code.to_string();
    if wf_mp_code == "".to_string()
    {
        wf_mp_code = machine().get_selected_m_profile();
    }

    let (_status, dbl_chk_records) = q_select(
        C_MACHINE_WALLET_FUNDS,
        vec!["wf_trx_hash"],
        vec![
            simple_eq_clause("wf_mp_code", &wf_mp_code),
            simple_eq_clause("wf_trx_hash", wf_trx_hash),
            ModelClause {
                m_field_name: "wf_o_index",
                m_field_single_str_value: &wf_o_index as &(dyn ToSql + Sync),
                m_clause_operand: "=",
                m_field_multi_values: vec![],
            },
        ],
        vec![],
        0,
        false);
    if dbl_chk_records.len() > 0
    {
        // maybe update!
        return true;
    } else {
        //insert
        let now_ = application().now();
        let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("wf_mp_code", &wf_mp_code as &(dyn ToSql + Sync)),
            ("wf_address", &wf_address as &(dyn ToSql + Sync)),
            ("wf_block_hash", &wf_block_hash as &(dyn ToSql + Sync)),
            ("wf_trx_type", &wf_trx_type as &(dyn ToSql + Sync)),
            ("wf_trx_hash", &wf_trx_hash as &(dyn ToSql + Sync)),
            ("wf_o_index", &wf_o_index as &(dyn ToSql + Sync)),
            ("wf_o_value", &wf_o_value as &(dyn ToSql + Sync)),
            ("wf_creation_date", &wf_creation_date as &(dyn ToSql + Sync)),
            ("wf_mature_date", &wf_mature_date as &(dyn ToSql + Sync)),
            ("wf_last_modified", &now_ as &(dyn ToSql + Sync)),
        ]);
        dlog(
            &format!("Going to insert new fund to wallet {:?}", &values),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);

        return q_insert(
            C_MACHINE_WALLET_FUNDS,
            &values,
            false);
    }
}

//old_name_was deleteFromFunds
pub fn delete_from_funds(
    wf_trx_hash: &CDocHashT,
    wf_o_index: COutputIndexT,
    wf_mp_code: &String) -> bool
{
    q_delete(
        C_MACHINE_WALLET_FUNDS,
        vec![
            simple_eq_clause("wf_mp_code", &wf_mp_code),
            simple_eq_clause("wf_trx_hash", wf_trx_hash),
            ModelClause {
                m_field_name: "wf_o_index",
                m_field_single_str_value: &wf_o_index as &(dyn ToSql + Sync),
                m_clause_operand: "=",
                m_field_multi_values: vec![],
            },
        ],
        true);

    return true;
}

//old_name_was deleteFromFunds
#[allow(unused, dead_code)]
pub fn delete_from_funds_by_trx(
    trx: &BasicTxDocument) -> bool
{
    let mut res: bool = true;
    let wf_mp_code = machine().get_selected_m_profile();
    for an_input in trx.get_inputs()
    {
        res &= delete_from_funds(
            &an_input.m_transaction_hash,
            an_input.m_output_index,
            &wf_mp_code,
        );
    }
    return res;
}
/*


void Wallet::removeRef(CCoinCodeT coin_code)
{
  String mp_code = CMachine::getSelectedMProfile();
  CLog::log("removing unused coin from machine_used_utxos (" + cutils::shortCoinRef(coin_code) + ")");
  DbModel::dDelete(
    stbl_machine_used_coins,
    {{"lu_mp_code", mp_code},
    {"lu_coin", coin_code}});
}

void Wallet::restorUnUsedUTXOs()
{
  String mp_code = CMachine::getSelectedMProfile();
  // retrtieve all marked as spen which are not recorded in DAG and markewd as used before than 2 cycle
  CDateT cDate = cutils::minutesBefore(CMachine::getCycleByMinutes());
  String q = " SELECT * FROM " + stbl_machine_used_coins+ " WHERE lu_mp_code='" + mp_code + "' AND lu_coin NOT IN (SELECT sp_coin FROM c_trx_spend) AND lu_insert_date<'" + cDate + "'";
  QueryRes res = DbModel::customQuery(
    "db_comen_blocks",
    q,
    {"lu_coin"},
    0,
    {},
    false,
    true);
  CLog::log("found unused coins: " + cutils::dumpIt(res.records));
  for (QVDicT row: res.records)
    removeRef(row["lu_coin"].to_string());
}


*/