use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, dlog, machine};
use crate::lib::custom_types::{ClausesT, CAddressT, QVDicT, QV2DicT, QVDRecordsT};
use crate::lib::database::abs_psql::{ModelClause, q_custom_query, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::{C_MACHINE_WALLET_ADDRESSES, C_MACHINE_WALLET_FUNDS};
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
    addresses: Vec<String>,
    mp_code: String,
    fields: Vec<&str>) -> (bool, QVDRecordsT)
{
    let mut clauses: ClausesT = vec![];
    if mp_code != constants::ALL.to_string()
    {
        clauses.push(simple_eq_clause("wa_mp_code", &mp_code));
    }

    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "wa_address",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for an_add in &addresses {
        c1.m_field_multi_values.push(an_add as &(dyn ToSql + Sync));
    }
    clauses.push(c1);

    let (_status, records) = q_select(
        C_MACHINE_WALLET_ADDRESSES,
        fields,
        clauses,
        vec![],
        0,
        true,
    );

    return (true, records);
}

//old_name_was getAddressesList
pub fn get_addresses_list(
    mp_code: &String,
    fields: Vec<&str>,
    calc_sum: bool) -> (QVDRecordsT, QV2DicT)
{
    let mut clauses: ClausesT = vec![];

    let mut mp_code = mp_code.clone();
    if mp_code == ""
    {
        mp_code = machine().get_selected_m_profile();
    }

    if mp_code != constants::ALL.to_string()
    {
        clauses.push(simple_eq_clause("wa_mp_code", &mp_code));
    }

    let (_status, records) = q_select(
        C_MACHINE_WALLET_ADDRESSES,
        fields,
        clauses,
        vec![],
        0,
        false);

    dlog(
        &format!("Wallet Addresses: {:?}", records),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    if !calc_sum
    {
        return (records, HashMap::new());
    }

    let now_ = application().now();
    let mut addresses_dict: QV2DicT = HashMap::new();
    let mut complete_query = format!(
        "SELECT wf_address, CAST(SUM(wf_o_value) AS varchar) AS mat_sum, COUNT(*) mat_count FROM {} \
        WHERE wf_mp_code=$1 AND wf_mature_date<$2 GROUP BY wf_address", C_MACHINE_WALLET_FUNDS);
    let params = vec![
        &mp_code as &(dyn ToSql + Sync),
        &now_ as &(dyn ToSql + Sync),
    ];

    let (_status, tmp_records) = q_custom_query(
        &complete_query,
        &params,
        false);
    dlog(
        &format!("Wallet Addresses funds: {:?}", tmp_records),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    for elm in &tmp_records
    {
        let add: CAddressT = elm["wf_address"].to_string();
        if !addresses_dict.keys().cloned().collect::<Vec<CAddressT>>().contains(&add)
        {
            let coins_info: QVDicT = HashMap::from([
                ("mat_sum".to_string(), elm["mat_sum"].clone()),
                ("mat_count".to_string(), elm["mat_count"].clone())
            ]);
            addresses_dict.insert(add, coins_info);
        }
    }

    // unmaturated coins
    complete_query = format!(
        "SELECT wf_address, SUM(wf_o_value) unmat_sum, COUNT(*) unmat_count FROM {} \
      WHERE wf_mp_code=$1 AND wf_mature_date >= $2 GROUP BY wf_address", C_MACHINE_WALLET_FUNDS);

    let now_ = application().now();
    let params = vec![
        &mp_code as &(dyn ToSql + Sync),
        &now_ as &(dyn ToSql + Sync),
    ];

    let (_status, records) = q_custom_query(
        &complete_query,
        &params,
        false);
    for elm in &records
    {
        let add: CAddressT = elm["wf_address"].to_string();
        if !addresses_dict
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .contains(&add)
        {
            let coins_info = HashMap::from([
                ("unmat_sum".to_string(), elm["unmat_sum"].clone()),
                ("unmat_count".to_string(), elm["unmat_count"].clone())
            ]);
            addresses_dict.insert(add, coins_info);
        } else {
            let coins_info = HashMap::from([
                ("unmat_sum".to_string(), elm["unmat_sum"].clone()),
                ("unmat_count".to_string(), elm["unmat_count"].clone())
            ]);
            addresses_dict.insert(add, coins_info);
        }
    }

    return (records, addresses_dict);
}

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
        vec![w_address.m_address.clone()],
        w_address.m_mp_code.clone(),
        vec!["wa_address"]);
    if !status
    { return (false, "Failed in search in wallet!".to_string()); }

    if addresses.len() > 0
    {
        return (false, "Adress already existed".to_string());
    }

    let (_status, values) = convert_to_values(w_address);
    dlog(
        &format!("Insert new address to machine wallet {:?}", dump_hashmap_of_str_string(&values)),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);
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
    let values_: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("wa_mp_code", &w_address.m_mp_code as &(dyn ToSql + Sync)),
        ("wa_address", &w_address.m_address as &(dyn ToSql + Sync)),
        ("wa_title", &w_address.m_title as &(dyn ToSql + Sync)),
        ("wa_creation_date", &w_address.m_creation_date as &(dyn ToSql + Sync)),
        ("wa_detail", &serialized_res as &(dyn ToSql + Sync)),
    ]);

    q_insert(
        C_MACHINE_WALLET_ADDRESSES,
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
    application().now()));

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
  the_address = wallet_controlled_accounts[rand() * wallet_controlled_accounts.len()]["wa_address"].to_string();
  return {(the_address != ""), the_address};
}

*/