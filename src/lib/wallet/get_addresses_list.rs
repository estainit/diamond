use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, dlog, machine};
use crate::lib::custom_types::{CAddressT, ClausesT, QV2DicT, QVDicT, QVDRecordsT};
use crate::lib::database::abs_psql::{q_custom_query, q_select, simple_eq_clause};
use crate::lib::database::tables::{C_MACHINE_WALLET_ADDRESSES, C_MACHINE_WALLET_FUNDS};

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

    let (_status, addresses_records) = q_select(
        C_MACHINE_WALLET_ADDRESSES,
        fields,
        clauses,
        vec![],
        0,
        false);

    dlog(
        &format!("Wallet Addresses: {:?}", addresses_records),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    if !calc_sum
    {
        return (addresses_records, HashMap::new());
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

    let (_status, mat_records) = q_custom_query(
        &complete_query,
        &params,
        false);
    dlog(
        &format!("Wallet Addresses funds: {:?}", mat_records),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    for elm in &mat_records
    {
        let the_address: CAddressT = elm["wf_address"].clone();
        if !addresses_dict.keys().cloned().collect::<Vec<CAddressT>>().contains(&the_address)
        {
            let coins_info: QVDicT = HashMap::from([
                ("mat_sum".to_string(), elm["mat_sum"].clone()),
                ("mat_count".to_string(), elm["mat_count"].clone())
            ]);
            addresses_dict.insert(the_address, coins_info);
        }
    }
    dlog(
        &format!("The addresses dict after extract maturated coins: {:?}", addresses_dict),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    // unmaturated coins
    complete_query = format!(
        "SELECT wf_address, CAST(SUM(wf_o_value) AS varchar) AS unmat_sum, COUNT(*) unmat_count FROM {} \
        WHERE wf_mp_code=$1 AND wf_mature_date >= $2 GROUP BY wf_address", C_MACHINE_WALLET_FUNDS);

    let now_ = application().now();
    let params = vec![
        &mp_code as &(dyn ToSql + Sync),
        &now_ as &(dyn ToSql + Sync),
    ];

    let (_status, un_mat_records) = q_custom_query(
        &complete_query,
        &params,
        false);
    for elm in &un_mat_records
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
    dlog(
        &format!("The addresses dict after extract un-maturated coins: {:?}", addresses_dict),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    return (addresses_records, addresses_dict);
}