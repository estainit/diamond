use std::collections::HashMap;
use postgres::types::ToSql;
use crate::cutils;
use crate::lib::custom_types::{ClausesT, LimitT, OrderT, QVDRecordsT};
use crate::lib::database::abs_psql::{q_select, q_upsert, simple_eq_clause};
use crate::lib::database::tables::STBL_KVALUE;

//old_name_was getValue
pub fn get_value(kv_key: &str) -> String
{
    let (status, records) = q_select(
        STBL_KVALUE,
        &vec!["kv_value"],
        vec![simple_eq_clause("kv_key", kv_key)],
        vec![],
        0,
        true,
    );
    println!("00000000kv val res:{:?}", records);

    if records.len() == 0 {
        return "".to_string();
    }
    return records[0].get("kv_value").unwrap().clone();
}

/*

bool KVHandler::deleteKey(const String &kvKey)
{
  QueryRes res = DbModel::dDelete(
    STBL_KVALUE,
    {{"kv_key", kvKey}}
  );
  return res.status == true;
}

*/
pub fn search_in_kv(
    clauses: ClausesT,
    fields: &Vec<&str>,
    order: OrderT,
    limit: LimitT) -> QVDRecordsT
{
    let (status, records) = q_select(
        STBL_KVALUE,
        fields,
        clauses,
        order,
        limit,
        true);
    return records;
}

/*
//  static prepareIt(args) {
//    args.table = 'i_kvalue';
//    let { _fields, _clauses, _values, _order, _limit } = db.pre_query_generator(args)
//    let _query = 'SELECT ' + _fields + ' FROM i_kvalue ' + _clauses + _order + _limit;
//    return { _query, _values }
//  }


bool KVHandler::updateKValue(const String &key, const String &value)
{
  return DbModel::update(
    STBL_KVALUE,
    {{"kv_value", value}, {"kv_last_modified", cutils::get_now()}},
    {{"kv_key", key}});
}

 */

pub fn upsert_kvalue(
    key: &str,
    value: &str,
    log: bool) -> bool
{
    let dt = cutils::get_now();
    let values: HashMap<&str, &(dyn ToSql + Sync)> =
        [("kv_value", &value as &(dyn ToSql + Sync)),
            ("kv_last_modified", &dt as &(dyn ToSql + Sync))]
            .iter().cloned().collect();

    return q_upsert(
        STBL_KVALUE,
        "kv_key",
        key,
        &values,
        log);
}

pub fn set_value(
    key: &str,
    value: &str,
    log: bool) -> bool
{
    return upsert_kvalue(key, value, log);
}


