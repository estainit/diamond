use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{constants, dlog};
use crate::lib::custom_types::{ClausesT, QVDRecordsT};
use crate::lib::database::abs_psql::{ModelClause, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::STBL_MACHINE_WALLET_ADDRESSES;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;
use crate::lib::utils::dumper::{dump_hashmap_of_str_string};

pub struct WalletAddress
{
    pub m_mp_code: String,
    // machine profile code
    pub m_address: String,
    pub m_title: String,
    pub m_unlock_doc: UnlockDocument,
    pub m_creation_date: String,

}

impl WalletAddress
{
    pub fn new(
        unlock_doc: &UnlockDocument,
        mp_code: String,
        title: String,
        creation_date: String,
    ) -> WalletAddress {
        return WalletAddress {
            m_mp_code: mp_code,
            m_address: unlock_doc.m_account_address.clone(),
            m_title: title,
            m_unlock_doc: unlock_doc.clone(),
            m_creation_date: creation_date,
        };
    }
}

//old_name_was searchWalletAdress
pub fn search_wallet_addresses(
    addresses: Vec<&str>,
    mp_code: String,
    fields: Vec<&str>) -> (bool, QVDRecordsT)
{
    let mut clauses: ClausesT = vec![];
    if mp_code != constants::ALL
    {
        clauses.push(simple_eq_clause("wa_mp_code", &*mp_code));
    }

    clauses.push(ModelClause {
        m_field_name: "wa_address",
        m_field_single_str_value: "",
        m_clause_operand: "IN",
        m_field_multi_values: addresses,
    });

    let (status, records) = q_select(
        STBL_MACHINE_WALLET_ADDRESSES,
        &fields,
        &clauses,
        vec![],
        0,
        true,
    );

    return (true, records);
}

/*
std::tuple<QVDRecordsT, QV2DicT> Wallet::getAddressesList(
    String mp_code,
    const StringList& fields,
    const bool& sum)
{
  ClausesT clauses{};

  if (mp_code == "")
    mp_code = CMachine::getSelectedMProfile();

  if (mp_code != constants::ALL)
    clauses.push(ModelClause("wa_mp_code", mp_code));

  QueryRes addresses_info = DbModel::select(
    stbl_machine_wallet_addresses,
    fields,
    clauses
  );

  if (sum == false)
    return {addresses_info.records, {}};

  CDateT nowT = cutils::get_now();
  QV2DicT addressDict = {};
  String complete_query = "select wf_address, SUM(wf_o_value) mat_sum, COUNT(*) mat_count FROM " + stbl_machine_wallet_funds + " ";
  complete_query += "WHERE wf_mp_code=:wf_mp_code AND wf_mature_date<:wf_mature_date GROUP BY wf_address";
  QueryRes tmpRes = DbModel::customQuery(
    "db_comen_wallets",
    complete_query,
    {"wf_address", "mat_sum", "mat_count"},
    0,
    {{"wf_mp_code", mp_code}, {"wf_mature_date", nowT}});

  for (QVDicT elm: tmpRes.records)
  {
    CAddressT add = elm.value("wf_address").to_string();
    if (!addressDict.keys().contains(add))
      addressDict[add] = QVDicT {
      {"mat_sum", elm.value("mat_sum").toDouble()},
      {"mat_count", elm.value("mat_count").toDouble()}};
  }

  // unmaturated coins
  complete_query = "SELECT wf_address, SUM(wf_o_value) unmat_sum, COUNT(*) unmat_count FROM " + stbl_machine_wallet_funds + " ";
  complete_query += "WHERE wf_mp_code=:wf_mp_code AND wf_mature_date >= :wf_mature_date GROUP BY wf_address";
  tmpRes = DbModel::customQuery(
    "db_comen_wallets",
    complete_query,
    {"wf_address", "unmat_sum", "unmat_count"},
    0,
    {{"wf_mp_code", mp_code}, {"wf_mature_date", cutils::get_now()}});

  for (QVDicT elm: tmpRes.records)
  {
    CAddressT add = elm.value("wf_address").to_string();
    if (!addressDict.keys().contains(add))
    {
      addressDict[add] = QVDicT {
      {"unmat_sum", elm.value("unmat_sum").toDouble()},
      {"unmat_count", elm.value("unmat_count").toDouble()}};
    }else{
      addressDict[add]["unmat_sum"] = elm.value("unmat_sum").toDouble();
      addressDict[add]["unmat_count"] = elm.value("unmat_count").toDouble();
    }
  }

  return {addresses_info.records, addressDict};

}
*/
//old_name_was convertToValues
pub fn convert_to_values(w_address: &WalletAddress) -> (bool, HashMap<&str, String>)
{
    let (status, serialized_res) = match serde_json::to_string(&w_address.m_unlock_doc) {
        Ok(ser) => { (true, ser) }
        Err(e) => {
            dlog(
                &format!("Failed in serialization m_unlock_doc {:?}", e),
                constants::Modules::App,
                constants::SecLevel::Error);
            (false, "".to_string())
        }
    };
    if !status
    {
        let r: HashMap<&str, String> = HashMap::new();
        return (false, r);
    }

    // let serialized_res = serialized_res.clone();
    let values: HashMap<&str, String> = HashMap::from([
        ("wa_mp_code", w_address.m_mp_code.clone()),
        ("wa_address", w_address.m_address.clone()),
        ("wa_title", w_address.m_title.clone()),
        ("wa_creation_date", w_address.m_creation_date.clone()),
        ("wa_detail", serialized_res),
    ]);
    return (true, values);
}


//old_name_was insertAddress
pub fn insert_address(w_address: &WalletAddress) -> (bool, String)
{
    let (status, addresses) = search_wallet_addresses(
        vec![&*w_address.m_address],
        w_address.m_mp_code.clone(),
        vec!["wa_address"]);
    if !status
    { return (false, "Failed in search in wallet!".to_string()); }

    if addresses.len() > 0
    {
        return (false, "Adress already existed".to_string());
    }

    let (status, values) = convert_to_values(w_address);
    dlog(
        &format!("Insert new address to machine wallet {:?}", dump_hashmap_of_str_string(&values)),
        constants::Modules::App,
        constants::SecLevel::Trace);
    // let mut values_: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::new();
    // for (k, v) in &values {
    //     values_.insert(*k, &v as &(dyn ToSql + Sync));
    // }

    let (status, serialized_res) = match serde_json::to_string(&w_address.m_unlock_doc) {
        Ok(ser) => { (true, ser) }
        Err(e) => {
            dlog(
                &format!("Failed in serialization m_unlock_doc {:?}", e),
                constants::Modules::App,
                constants::SecLevel::Error);
            (false, "".to_string())
        }
    };
    if !status {
        return (false, "Failed in serialization m_unlock_doc".to_string());
    }
    let mut values_: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("wa_mp_code", &w_address.m_mp_code as &(dyn ToSql + Sync)),
        ("wa_address", &w_address.m_address as &(dyn ToSql + Sync)),
        ("wa_title", &w_address.m_title as &(dyn ToSql + Sync)),
        ("wa_creation_date", &w_address.m_creation_date as &(dyn ToSql + Sync)),
        ("wa_detail", &serialized_res as &(dyn ToSql + Sync)),
    ]);

    q_insert(
        STBL_MACHINE_WALLET_ADDRESSES,
        &values_,
        true,
    );
    return (true, "Inserted new address to machine wallet".to_string());
}

/*

QVDRecordsT Wallet::getAddressesInfo(
  const StringList& addresses,
  const StringList& fields)
{
  String mp_code = CMachine::getSelectedMProfile();
  auto[status, res] = search_wallet_addresses(addresses, mp_code, fields);
  if (!status)
      return {};
  return res;
}

GenRes Wallet::createANewAddress(
  const String& signature_type,
  const String& signature_mod,
  const String& signature_version)
{
  auto[status, unlock_doc] = CAddress::createANewAddress(
    signature_type,
    signature_mod,
    signature_version);
  if (!status)
    return {false, ""};// {false, "Couldn't creat ECDSA key pairs (for public channel)"};

  insertAddress( WalletAddress (
    &unlock_doc,
    CMachine::getSelectedMProfile(),   // mp code
    signature_type + " address (" + signature_mod + " signatures) ver(" + signature_version + ")",
    cutils::get_now()));

  CGUI::signalUpdateWalletCoins();
  CGUI::signalUpdateWalletAccounts();

  return {true, unlock_doc.m_account_address};
}

GenRes Wallet::getAnOutputAddress(
  bool make_new_address,
  const String& signature_type,
  const String& signature_mod,
  const String& signature_version)
{
  CAddressT the_address;
  if (make_new_address)
  {
    return createANewAddress(signature_type, signature_mod, signature_version);
  }

  auto[wallet_controlled_accounts, details] = getAddressesList();
  the_address = wallet_controlled_accounts[rand() * wallet_controlled_accounts.len()].value("wa_address").to_string();
  return {(the_address != ""), the_address};
}

*/