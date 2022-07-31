
use std::collections::HashMap;
use crate::cutils;
use crate::lib::database::abs_psql::{ModelClause, q_select, q_upsert};
use crate::lib::database::tables::STBL_KVALUE;

//old_name_was getValue
pub fn get_value(kv_key: &String) -> String
{
    let (status, records) = q_select(
        STBL_KVALUE,
        &vec!["kv_value"],
        &vec![
            &ModelClause {
                m_field_name: "kv_key",
                m_field_single_str_value: kv_key,
                m_clause_operand: "=",
                m_field_multi_values: vec![],
            }],
        &vec![],
        0,
        true,
    );
    println!("00000000kv val res:{:?}", records);

    if records.len() == 0 {
        return "".to_string();
    }
    return "iiiiiiii".to_string();
    // return res.records[0].value("kv_value").to_string();
}

/*

bool KVHandler::deleteKey(const QString &kvKey)
{
  QueryRes res = DbModel::dDelete(
    STBL_KVALUE,
    {{"kv_key", kvKey}}
  );
  return res.status == true;
}

QVDRecordsT KVHandler::serach(
  const ClausesT& clauses,
  const QStringList& fields,
  const OrderT& order,
  const int& limit)
{
  try {
    QueryRes res = DbModel::select(
      STBL_KVALUE,
      fields,
      clauses,
      order,
      limit,
      false,
      true);
    return res.records;

  } catch (std::exception) {
    CLog::log("faile in search in kvhandler", "app", "fatal");
    return {};
  }
}

//  static prepareIt(args) {
//    args.table = 'i_kvalue';
//    let { _fields, _clauses, _values, _order, _limit } = db.pre_query_generator(args)
//    let _query = 'SELECT ' + _fields + ' FROM i_kvalue ' + _clauses + _order + _limit;
//    return { _query, _values }
//  }


bool KVHandler::updateKValue(const QString &key, const QString &value)
{
  return DbModel::update(
    STBL_KVALUE,
    {{"kv_value", value}, {"kv_last_modified", CUtils::getNow()}},
    {{"kv_key", key}});
}

 */

pub fn upsert_kvalue(
    key: &str,
    value: &str,
    log: bool) -> bool
{
    let dt = cutils::get_now();
    let values: HashMap<&str, &str> =
        [("kv_value", value),
            ("kv_last_modified", &dt)]
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


