use std::borrow::Borrow;
use std::collections::HashMap;
use postgres::Row;
use postgres::row::RowIndex;
use postgres::types::{FromSql, ToSql};
use crate::{cutils, dbhandler, machine};
use crate::cutils::remove_dbl_spaces;
use crate::lib::constants;
use crate::lib::custom_types::{ClausesT, OrderT, QVDicT, VString};
use crate::lib::dlog::dlog;
use crate::lib::utils::dumper::dump_clauses;

pub mod db_model {
    pub const S_SINGLE_OPERANDS: [&str; 14] = ["=", "<", ">", "<=", "=<", ">=", "like", "LIKE", "ilike", "ILIKE", "not like", "NOT LIKE", "not ilike", "NOT ILIKE"];
}

pub struct QueryElements {
    pub m_clauses: String,
    pub m_params: Vec<String>,
    pub m_order: String,
    pub m_limit: String,
    pub m_complete_query: String,
}

pub struct OrderModifier<'l>
{
    pub m_field: &'l str,
    pub m_order: &'l str,
    // OrderModifier( const QString & field, const QString & order);
}

pub struct ModelClause<'l>
{
    pub m_field_name: &'l str,
    // "=";
    pub m_field_single_str_value: &'l str,
    pub m_clause_operand: &'l str,
    // "=";
    pub m_field_multi_values: Vec<&'l str>,

    // ModelClause(const QString & fieldName, const QVariant & fieldValue);
    // ModelClause(const QString & fieldName, const QVariant & fieldValue, const QString & clause_operand);
    // ModelClause(const QString & fieldName, const QStringList & fieldValues, const QString & clause_operand = "IN");
}

// impl fmt::Debug for ModelClause<'static> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let rr = format!("{:?}", &self.m_field_multi_values);
//         f.debug_struct("Point")
//             .field("m_field_name", &self.m_field_name)
//             .field("m_field_single_str_value", &self.m_field_single_str_value)
//             .field("m_clause_operand", &self.m_clause_operand)
//             .finish()
//     }
// }

pub struct ModelParams<'l>
{
    pub m_table: &'l str,
    pub m_clauses: ClausesT<'l>,
    pub m_fields: Vec<&'l str>,
    pub m_order: OrderT<'l>,
    pub m_limit: i8,
}

// pub struct PTRRes
// {
//     pub complete_query: String,
//     pub query_elements: QueryElements, //old_name_was qElms
// }

// pub struct QueryRes
// {
//     pub status: bool,
//     pub records: Vec<Row>,
// }

/*
ModelClause::ModelClause(const QString& field_name, const QVariant& field_value, const QString& clause_operand)
{
  m_field_name = field_name;
  m_field_single_str_value = field_value;
  m_clause_operand = clause_operand;
}

ModelClause::ModelClause(const QString& field_name, const QVariant& field_value)
{
  m_field_name = field_name;
  m_field_single_str_value = field_value;
}

ModelClause::ModelClause(const QString& field_name, const QStringList& field_values, const QString& clause_operand)
{
  m_field_name = field_name;
  m_field_single_str_value = "";
  m_field_multi_values = field_values;
  m_clause_operand = clause_operand;
}


//  -  -  -  DbModel part
const QSDicT DbModel::s_map_table_to_db = {
  {"c_blocks", "db_comen_blocks"},
  {"c_docs_blocks_map", "db_comen_blocks"},
  {"c_block_extinfos", "db_comen_block_ext_info"},
  {"c_trx_utxos", "db_comen_spendable_coins"},
  {"c_trx_spend", "db_comen_spent_coins"},
  {"c_machine_wallet_addresses", "db_comen_wallets"},
  {"c_machine_wallet_funds", "db_comen_wallets"},
};

*/

#[derive(Debug)]
pub struct QUnion {
    pub i: i64,
    pub f: f64,
    pub b: bool,
    pub s: String,
}

pub fn q_select(
    table: &str,
    fields: &Vec<&str>,
    clauses: &ClausesT,
    order: &OrderT,
    limit: i8,
    do_log: bool) -> (bool, Vec<HashMap<String, QUnion>>)
{
    return exec_query(
        &prepare_to_select(table, fields, clauses, order, limit),
        clauses,
        fields,
        &HashMap::new(),
        do_log);//, lockDb, log);
}

pub fn prepare_to_select(
    table: &str,
    fields: &Vec<&str>,
    clauses: &ClausesT,
    order: &OrderT,
    limit: i8) -> QueryElements
{
    let mut query_elements: QueryElements = pre_query_generator(0, clauses, order, limit);
    let mut complete_query: String = "SELECT ".to_owned() + &fields.join(", ") + " FROM " + table + &query_elements.m_clauses + &query_elements.m_order + &query_elements.m_limit;
    complete_query = complete_query.trim().parse().unwrap();
    query_elements.m_complete_query = remove_dbl_spaces(&complete_query);
    return query_elements;
}

/*
QueryRes DbModel::dDelete(
  const QString& table,
  const ClausesT& clauses,
  const bool& is_transactional,
  const bool& do_log)
{
  QString database_name = s_map_table_to_db.keys().contains(table) ? s_map_table_to_db[table] : "db_comen_general";

  PTRRes r = prepareToDelete(table, clauses);
  return DbModel::exec_query(database_name, r.complete_query, clauses, {}, {}, is_transactional, do_log);
}

PTRRes DbModel::prepareToDelete(
  const QString& table,
  const ClausesT& clauses)
{
  QueryElements query_elements = pre_query_generator(clauses);
  QString complete_query = "DELETE FROM " + table + query_elements.m_clauses + query_elements.m_order + query_elements.m_limit;
  complete_query = complete_query.trimmed();
  complete_query = CUtils::removeDblSpaces(complete_query);
  return {complete_query, query_elements};
}
*/

pub fn clauses_query_generator(
    placeholder_offset: u8,
    clauses: &ClausesT,
) -> (String, Vec<String>)
{
    println!("in clauses_ query_ generator clauses: {:?}", dump_clauses(clauses));

    let mut values_: Vec<String> = vec![];
    let mut clauses_: String = "".to_string();
    let val_arr: Vec<String>;

    if clauses.len() > 0 {
        let mut tmp: Vec<String> = vec![];

        let mut placeholder_index = placeholder_offset;
        for a_clause_tuple in clauses
        {
            placeholder_index += 1;
            let mut key = a_clause_tuple.m_field_name;
            if db_model::S_SINGLE_OPERANDS.contains(&a_clause_tuple.m_clause_operand)
            {
                tmp.push(" (".to_owned() + &key + &" ".to_owned() + &a_clause_tuple.m_clause_operand + &format!(" ${placeholder_index} ) "));
                values_.push(a_clause_tuple.m_field_single_str_value.parse().unwrap());
            } else if (a_clause_tuple.m_clause_operand == "IN") || (a_clause_tuple.m_clause_operand == "NOT IN")
            {
                if a_clause_tuple.m_field_multi_values.len() == 0
                {
                    dlog(
                        &format!("Maleformed clauses 'IN/NOT IN': {:?}", dump_clauses(clauses)),
                        constants::Modules::Sql,
                        constants::SecLevel::Fatal);
                    panic!("{}", format!("Maleformed clauses 'IN/NOT IN': {:?}", dump_clauses(clauses)));
                }

                let mut tmp_placeholders: Vec<String> = vec![];
                for i in 0..a_clause_tuple.m_field_multi_values.len()
                {
                    placeholder_index += 1;
                    tmp_placeholders.push(format!("${}", placeholder_index));
                    values_.push(a_clause_tuple.m_field_multi_values[i].clone().to_string());
                }
                tmp.push(" (".to_owned() + &key + &" ".to_owned() + &a_clause_tuple.m_clause_operand + &" (".to_owned() + &tmp_placeholders.join(", ") + ")) ");
            } else if a_clause_tuple.m_clause_operand == "LIKE:OR"
            {
                panic!("LIKE:OR NOT IMPLEMENTED YET!");
            }
        }


        if (tmp.len() > 0) && (values_.len() > 0) {
            clauses_ = tmp.join(" AND ");
        }
    }

    return (clauses_, values_);
}
/*

*/

pub fn pre_query_generator(
    placeholder_offset: u8,
    clauses_: &ClausesT,
    order_: &OrderT,
    limit_: i8) -> QueryElements
{
    let (mut clauses, values_) = clauses_query_generator(placeholder_offset, clauses_);
    if clauses.len() > 0 {
        clauses = " WHERE ".to_owned() + &clauses;
    }

    let mut order: String = "".to_string();
    if order_.len() > 0 {
        let mut orders: Vec<String> = vec![];
        for an_order in order_ {
            orders.push(an_order.m_field.to_owned() + " " + &an_order.m_order);
        }
        order = " ORDER BY ".to_owned() + &orders.join(", ");
    }

    let mut limit: String = "".to_string();
    if limit_ > 0 {
        limit = format!(" LIMIT {} ", limit_);
    }

    return QueryElements {
        m_clauses: clauses,
        m_params: values_,
        m_order: order,
        m_limit: limit,
        m_complete_query: "".to_string(),
    };
}
/*

QueryRes DbModel::customQuery(
  const QString& database_name,
  const QString& complete_query,
  const QStringList& fields,
  const int& field_count,
  const QVDicT& to_bind_values,
  const bool& is_transactional,
  const bool& do_log)
{
  if (CConsts::DATABASAE_AGENT == "psql")
    return customPSQLQuery(database_name, complete_query, fields, field_count, to_bind_values, is_transactional, do_log);

  Q_UNUSED(is_transactional);
  QSqlQuery query(DbHandler::getDB(database_name));
  query.setForwardOnly(true);
  if (do_log)
    CLog::log(complete_query, "sql", "trace");

  query.prepare(complete_query);

  for (QString aKey : to_bind_values.keys())
    query.bindValue(":"+aKey, to_bind_values[aKey]);

  QMap<QString, QVariant> sqlIterator(query.boundValues());
  QStringList bound_list_for_log;
  for (auto i = sqlIterator.begin(); i != sqlIterator.end(); ++i)
  {
    bound_list_for_log.push(i.key().toUtf8() + ": " + i.value().to_string().toUtf8());
  }
  if (do_log)
    CLog::log("Query Values: [" + bound_list_for_log.join(", ") + "]", "sql", "trace");

  QVDRecordsT records;
  if (!query.exec())
  {
    QString err_text = query.lastError().databaseText();
    if (uniqueConstraintFailed(err_text))
      return {true, records};

    CLog::log(err_text, "sql", "fatal");
    CLog::log("SQL failed query1: " + complete_query, "sql", "fatal");
    CLog::log("SQL failed values1: " + CUtils::dumpIt(to_bind_values), "sql", "fatal");
    QString executed_query = query.executedQuery();
    CLog::log("SQL failed executed_query: " + executed_query, "sql", "fatal");
    CLog::log("SQL failed clauses2: " + err_text, "sql", "fatal");

    bool res = handleMaybeDbLock(query);
    if (!res)
      return {false, records};
  }

  while (query.next())
  {
    QVDicT a_row;
    if (fields.size() > 0)
    {
      for ( QString a_filed: fields)
      {
        a_row[a_filed] = query.value(a_filed);
      }

    }else{
      // return results by position
      for(int i=0; i < field_count; i++)
        a_row[QString::number(i)] = query.value(i);

    }
    records.push(a_row);

    QMap<QString, QVariant> sqlIterator(query.boundValues());
    QStringList bound_list_for_log;
    for (auto i = sqlIterator.begin(); i != sqlIterator.end(); ++i)
    {
      bound_list_for_log.push(i.key().toUtf8() + ": " + i.value().to_string().toUtf8());
    }
  }

  if (query.isSelect())
  {
    if (do_log)
      CLog::log("Results: [" + QString("%1").arg(records.size()) + " rows] returned", "sql", "trace");
  }
  else if (query.numRowsAffected() > 0)
  {
    CLog::log("Affected: [" + QString("%1").arg(query.numRowsAffected()) + " rows] afected", "sql", "trace");
  }


  query.finish();

  return {true, records};
}

QueryRes DbModel::customPSQLQuery(
  const QString& database_name,
  const QString& complete_query,
  const QStringList& fields,
  const int& field_count,
  const QVDicT& to_bind_values,
  const bool& is_transactional,
  const bool& do_log)
{
  Q_UNUSED(is_transactional);
  QSqlQuery* query = new QSqlQuery(*DbHandler::getPSQLDB(database_name));
  query->setForwardOnly(true);

  if (do_log)
    CLog::log(complete_query, "sql", "trace");

  QStringList bound_list_for_log;
  bool exec_res;
  if (to_bind_values.size() > 0)
  {
    query->prepare(complete_query);
    for (QString aKey : to_bind_values.keys())
      query->addBindValue(to_bind_values[aKey]);

    QMap<QString, QVariant> sqlIterator(query->boundValues());
    for (auto i = sqlIterator.begin(); i != sqlIterator.end(); ++i)
    {
      bound_list_for_log.push(i.key().toUtf8() + ": " + i.value().to_string().toUtf8());
    }

    exec_res = query->exec();

  } else {
    exec_res = query->exec(complete_query);

  }


  if (do_log)
    CLog::log("Query Values: [" + bound_list_for_log.join(", ") + "]", "sql", "trace");

  QVDRecordsT records;
  if (!exec_res)
  {
    QString err_text = query->lastError().databaseText();
    if (uniqueConstraintFailed(err_text))
      return {true, records};

    CLog::log(err_text, "sql", "fatal");
    CLog::log("PSQL failed query1: " + complete_query, "sql", "fatal");
    CLog::log("PSQL failed values1: " + CUtils::dumpIt(to_bind_values), "sql", "fatal");
    QString executed_query = query->executedQuery();
    CLog::log("PSQL failed executed_query: " + executed_query, "sql", "fatal");
    CLog::log("PSQL failed clauses2: " + err_text, "sql", "fatal");
    return {false, records};
  }

  while (query->next())
  {
    QVDicT a_row;
    if (fields.size() > 0)
    {
      for ( QString a_filed: fields)
      {
        a_row[a_filed] = query->value(a_filed);
      }

    }else{
      // return results by position
      for(int i=0; i < field_count; i++)
        a_row[QString::number(i)] = query->value(i);

    }
    records.push(a_row);

    QMap<QString, QVariant> sqlIterator(query->boundValues());
//    QStringList bound_list_for_log;
//    for (auto i = sqlIterator.begin(); i != sqlIterator.end(); ++i)
//      bound_list_for_log.push(i.key().toUtf8() + ": " + i.value().to_string().toUtf8());

  }

  if (query->isSelect())
  {
    if (do_log)
      CLog::log("Results: [" + QString("%1").arg(records.size()) + " rows] returned", "sql", "trace");
  }
  else if (query->numRowsAffected() > 0)
  {
    CLog::log("Affected: [" + QString("%1").arg(query->numRowsAffected()) + " rows] afected", "sql", "trace");
  }

  query->finish();
  delete query;

  return {true, records};
}

*/

pub fn exec_query(
    query_elements: &QueryElements,
    clauses: &ClausesT,
    fields: &Vec<&str>,
    upd_values: &HashMap<&str, &str>,
    do_log: bool) -> (bool, Vec<HashMap<String, QUnion>>)
{
    if do_log {
        dlog(
            &query_elements.m_complete_query.to_string(),
            constants::Modules::Sql,
            constants::SecLevel::Trace);
    }

    if do_log {
        dlog(
            &format!("Query Values: [{}] ", query_elements.m_params.join(", ")),
            constants::Modules::Sql,
            constants::SecLevel::Trace);
    }
    let mut out_rows: Vec<HashMap<String, QUnion>> = vec![];
    let params: Vec<_> = query_elements.m_params.iter().map(|x| x as &(dyn ToSql + Sync)).collect();
    return match dbhandler().m_db.query(
        &query_elements.m_complete_query,
        &params[..]) {
        Ok(rows) => {
            if do_log {
                dlog(
                    &format!("Query executed successfully: "),
                    constants::Modules::Sql,
                    constants::SecLevel::Trace);
            }

            if rows.len() == 0
            {
                let res: Vec<Row> = vec![];
                return (true, out_rows);
            }

            let rr = Row::columns(&rows[0]);
            // println!("a rr rr rr: {:?}", rr);
            // println!("a rr rr rr[0]: {:?}", rr[0]);
            // println!("a rr rr rr[0]type_: {:?}", rr[0].type_());
            let mut res_cols_info: Vec<(String, String)> = vec![];
            for a_col in &*rows[0].columns() {
                println!("a col: {:?}", a_col);
                // println!("a col.name: {:?}", a_col.name());
                res_cols_info.push((a_col.name().to_string(), a_col.type_().to_string()));
            }
            // println!("a res_col_names: {:?}", res_col_names);


            for mut a_row in &rows {
                let mut a_row_dict: HashMap<String, QUnion> = HashMap::new();
                for col_inx in 0..a_row.len() {
                    let (col_name, col_type) = &res_cols_info[col_inx];
                    // let col_name = res_cols_info[col_inx].clone();
                    let col_value: QUnion = match &*col_type.clone() {
                        ("real" | "double precision") => {
                            let col_value: f64 = Row::get(a_row, col_inx);
                            QUnion {
                                i: 0,
                                f: col_value,
                                b: false,
                                s: "".to_string(),
                            }
                        }
                        ("smallint" | "smallserial" | "int" | "serial" | "oid" | "bigint" | "bigserial") => {
                            let col_value: i64 = Row::get(a_row, col_inx);
                            QUnion {
                                i: col_value,
                                f: 0.0,
                                b: false,
                                s: "".to_string(),
                            }
                        }
                        ("varchar") => {
                            let col_value: String = Row::get(a_row, col_inx);
                            QUnion {
                                i: 0,
                                f: 0.0,
                                b: false,
                                s: col_value,
                            }
                        }
                        ("text") => {
                            let col_value: String = Row::get(a_row, col_inx);
                            QUnion {
                                i: 0,
                                f: 0.0,
                                b: false,
                                s: col_value,
                            }
                        }
                        ("bool") => {
                            let col_value: bool = Row::get(a_row, col_inx);
                            QUnion {
                                i: 0,
                                f: 0.0,
                                b: col_value,
                                s: "".to_string(),
                            }
                        }
                        (_) => {
                            let col_value: String = Row::get(a_row, col_inx);
                            println!("UUUUU Unsetted col type {} {} {}", col_type.clone(), col_name.clone(), col_value.clone());
                            QUnion {
                                i: 0,
                                f: 0.0,
                                b: false,
                                s: col_value,
                            }
                        }
                    };
                    a_row_dict.insert(col_name.clone(), col_value);
                }
                out_rows.push(a_row_dict);
            }
            println!(">>> out_dict: {:?}", out_rows);
            (true, out_rows)
        }
        Err(e) => {
            dlog(
                &format!("Failed in Q e: {:?} ", e),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            dlog(
                &format!("Failed in Q query: {} ", query_elements.m_complete_query),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            dlog(
                &format!("Failed in Q params: [{}] ", query_elements.m_params.join(", ")),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            (true, vec![])
        }
    };
}

/*

bool DbModel::handleMaybeDbLock(QSqlQuery& query)
{
  QString dummeyLockErrMsg = "database is locked";
  if (query.lastError().databaseText() != dummeyLockErrMsg)
    return false;

  while(true)
  {
    std::this_thread::sleep_for(std::chrono::seconds(1));
    query.finish();
    if (query.exec())
    {
      return true;

    }else{
      CLog::log("handle Maybe Db Lock failed Thread: " + QString::number((quint64)QThread::currentThread(), 16), "sql", "fatal");
      CLog::log("handle Maybe Db Lock" + query.lastError().databaseText(), "sql", "fatal");
      QString executed_query = query.executedQuery();
      CLog::log("SQL failed executed_query: " + executed_query, "sql", "fatal");
      QString err_text = query.lastError().databaseText();
      CLog::log("SQL failed clauses2: " + err_text, "sql", "fatal");
      if (err_text != dummeyLockErrMsg)
        return false;
    }
  }

  CLog::log("even handle Maybe DbLock didn't work!", "app", "fatal");
  return false;
}


const QString UNIQUE_ERR_MSG_PREFIX1 = "UNIQUE constraint failed";
const QString UNIQUE_ERR_MSG_PREFIX2 = "ERROR:  duplicate key value violates unique constraint";

bool DbModel::uniqueConstraintFailed(const QString& msg)
{
  if (msg.midRef(0, UNIQUE_ERR_MSG_PREFIX1.length()).to_string() == UNIQUE_ERR_MSG_PREFIX1)
    return true;
  if (msg.midRef(0, UNIQUE_ERR_MSG_PREFIX2.length()).to_string() == UNIQUE_ERR_MSG_PREFIX2)
    return true;
  return false;
}

*/
pub fn q_insert(
    table: &str,
    values: &HashMap<&str, &str>,
    do_log: bool) -> bool
{
    return insert_to_psql(table, values, do_log);
}


pub fn insert_to_psql(
    table: &str,
    values: &HashMap<&str, &str>,
    do_log: bool) -> bool
{
    let mut position: u8 = 0;
    let mut placeholders: Vec<String> = vec![];
    let mut the_keys: Vec<&str> = vec![];
    let mut params: Vec<&str> = vec![];
    for (k, v) in values {
        position += 1;
        let pos = format!("${}", position);
        placeholders.push(pos);
        the_keys.push(k);
        params.push(v);
    }
    // let keys = values.iter().map(|(&k, &v)| k).collect::<Vec<_>>(); //values.keys();
    let placeholders = placeholders.join(", ");
    let the_keys = the_keys.join(", ");
    let mut complete_query: String = "INSERT INTO ".to_owned() + table + " (" + &*the_keys + ") VALUES (" + &*placeholders + "); "; //BEGIN; COMMIT;

    if do_log {
        dlog(
            &format!("{}", complete_query),
            constants::Modules::Sql,
            constants::SecLevel::Info);
    }

    let log_values: Vec<&str> = vec![];

    let params: Vec<_> = params.iter().map(|x| x as &(dyn ToSql + Sync)).collect();
    return match dbhandler().m_db.execute(&complete_query, &params) {
        Ok(v) => { true }
        Err(e) => {
            dlog(
                &format!("failed in insert {:?}", e),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            dlog(
                &format!("Query: {:?}", complete_query),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            dlog(
                &format!("Params: {:?}", params),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            false
        }
    };
}

pub fn q_upsert(
    table: &str,
    controlled_field: &str,
    controlled_value: &str,
    values: &HashMap<&str, &str>,
    do_log: bool) -> bool
{
    let only_clause = ModelClause {
        m_field_name: controlled_field,
        m_field_single_str_value: controlled_value,
        m_clause_operand: "=",
        m_field_multi_values: vec![],
    };
    let clauses: ClausesT = vec![&only_clause];

    // controll if the record already existed
    let (status, records) = q_select(
        table,
        &vec![controlled_field],     // fields
        &clauses, //clauses
        &vec![],   // order
        1,   // limit
        do_log,
    );
    if !status
    { return false; }

    if records.len() > 0 {
        // update existed record
        q_update(
            table,
            values, // update values
            &clauses,  // update clauses
            do_log);

        return true;
    }

    // insert new entry
    let mut insert_values = values.clone();
    insert_values.insert(controlled_field, controlled_value);
    return q_insert(
        table,     // table
        &insert_values, // values to insert
        true,
    );
}

pub fn prepare_to_update(
    table: &str,
    upd_values: &HashMap<&str, &str>,
    clauses: &ClausesT) -> QueryElements
{
    let mut query_elements: QueryElements = pre_query_generator(upd_values.len() as u8, clauses, &vec![], 0);

    let mut updates: Vec<String> = vec![];
    let mut position = 0;
    let mut finall_upd_values: Vec<String> = vec![]; //query_elements.m_params
    for (a_key, a_value) in upd_values
    {
        position += 1;
        println!("the_clauseZ aKey: {}", a_key);
        let the_key = a_key.clone();
        let the_single_update = (the_key.to_owned() + &format!("= ${}", position)).clone();
        println!("the_clauseZ ff: {}", the_single_update);
        updates.push(the_single_update);
        finall_upd_values.push(a_value.to_string());
    }
    for a_value in &query_elements.m_params {
        finall_upd_values.push(a_value.to_string());
    }
    query_elements.m_params = finall_upd_values;

    let set_fields = updates.join(", ");
    let mut complete_query = "UPDATE ".to_owned() + &table + " SET " + &set_fields + &query_elements.m_clauses;
    complete_query = complete_query.trim().parse().unwrap();
    query_elements.m_complete_query = remove_dbl_spaces(&complete_query);

    return query_elements;
}

pub fn q_update(
    table: &str,
    update_values: &HashMap<&str, &str>,
    update_clauses: &ClausesT,
    do_log: bool) -> bool
{
    let query_elements = prepare_to_update(table, update_values, update_clauses);
    let upd_res = exec_query(
        &query_elements,
        update_clauses,
        &vec![],
        update_values,
        do_log);

    return true;
}
/*


    bool DbModel::clauseContains(const ClausesT& clauses, const QString& filed_name)   //legacy queryHas
    {
      for (ModelClause a_clause : clauses)
      {
        if (a_clause.m_field_name == filed_name)
          return true;
       }
       return false;
    }










     */