use std::collections::HashMap;
use postgres::Row;
use postgres::types::{ToSql, Type, FromSql};
use crate::{cutils, dbhandler};
use crate::cutils::{convert_float_to_string, remove_dbl_spaces};
use crate::lib::constants;
use crate::lib::custom_types::{ClausesT, LimitT, OrderT, QVDRecordsT};
use crate::lib::dlog::dlog;
use crate::lib::utils::dumper::dump_clauses;

pub mod db_model {
    pub const S_SINGLE_OPERANDS: [&str; 14] = ["=", "<", ">", "<=", "=<", ">=", "like", "LIKE", "ilike", "ILIKE", "not like", "NOT LIKE", "not ilike", "NOT ILIKE"];
}

pub struct QueryElements<'e> {
    pub m_clauses: String,
    pub m_params: Vec<&'e (dyn ToSql + Sync)>,
    pub m_order: String,
    pub m_limit: String,
    pub m_complete_query: String,
}

pub struct OrderModifier<'l>
{
    pub m_field: &'l str,
    pub m_order: &'l str,
    // OrderModifier( const String & field, const String & order);
}

#[derive(Clone)]
pub struct ModelClause<'l>
{
    pub m_field_name: &'l str,
    // "=";
    pub m_field_single_str_value: &'l str,
    pub m_clause_operand: &'l str,
    // "=";
    pub m_field_multi_values: Vec<&'l str>,

    // ModelClause(const String & fieldName, const QVariant & fieldValue);
    // ModelClause(const String & fieldName, const QVariant & fieldValue, const String & clause_operand);
    // ModelClause(const String & fieldName, const StringList & fieldValues, const String & clause_operand = "IN");
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

pub fn simple_eq_clause<'s>(field_name: &'s str, single_value: &'s str) -> ModelClause<'s> {
    return ModelClause {
        m_field_name: field_name.clone(),
        m_field_single_str_value: single_value.clone(),
        m_clause_operand: "=",
        m_field_multi_values: vec![],
    };
}

pub struct ModelParams<'l>
{
    pub m_table: &'l str,
    pub m_clauses: ClausesT<'l>,
    pub m_fields: Vec<&'l str>,
    pub m_order: OrderT<'l>,
    pub m_limit: LimitT,
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
ModelClause::ModelClause(const String& field_name, const QVariant& field_value, const String& clause_operand)
{
  m_field_name = field_name;
  m_field_single_str_value = field_value;
  m_clause_operand = clause_operand;
}

ModelClause::ModelClause(const String& field_name, const QVariant& field_value)
{
  m_field_name = field_name;
  m_field_single_str_value = field_value;
}

ModelClause::ModelClause(const String& field_name, const StringList& field_values, const String& clause_operand)
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

// #[derive(Debug)]
// pub struct QUnion {
//     pub su: String, // selected member
//     pub i: i64,
//     pub f: f64,
//     pub b: bool,
//     pub s: String,
// }
//
// impl QUnion{
//     pub fn from_string(s:String)->QUnion{
//         return QUnion{
//             su: "".to_string(),
//             i: 0,
//             f: 0.0,
//             b: false,
//             s
//         };
//
//     }
// }

pub fn q_select(
    table: &str,
    fields: &Vec<&str>,
    clauses: ClausesT,
    order: OrderT,
    limit: LimitT,
    do_log: bool) -> (bool, QVDRecordsT)
{
    return exec_query(
        &prepare_to_select(table, fields, &clauses, order, limit),
        do_log);//, lockDb, log);
}

pub fn prepare_to_select<'e>(
    table: &'e str,
    fields: &'e Vec<&str>,
    clauses: &'e ClausesT,
    order: OrderT,
    limit: LimitT) -> QueryElements<'e>
{
    let mut query_elements: QueryElements = pre_query_generator(0, clauses, order, limit);
    let mut complete_query: String = "SELECT ".to_owned() + &fields.join(", ") + " FROM " + table + &query_elements.m_clauses + &query_elements.m_order + &query_elements.m_limit;
    complete_query = complete_query.trim().parse().unwrap();
    query_elements.m_complete_query = remove_dbl_spaces(&complete_query);
    return query_elements;
}

pub fn q_delete(
    table: &str,
    clauses: ClausesT,
    do_log: bool) -> bool
{
    let (_complete_query, query_elements) = prepareToDelete(table, &clauses);
    let (status, _records) = exec_query(&query_elements, do_log);
    return status;
}

pub fn prepareToDelete<'e>(
    table: &'e str,
    clauses: &'e ClausesT) -> (String, QueryElements<'e>)
{
    let ord_ = vec![];
    let query_elements: QueryElements = pre_query_generator(0, clauses, ord_, 0);
    let mut complete_query: String = "DELETE FROM ".to_owned() + table + &query_elements.m_clauses + &query_elements.m_order + &query_elements.m_limit;
    // complete_query = complete_query.trim().parse().unwrap();
    // complete_query = cutils::removeDblSpaces(complete_query);
    return (complete_query, query_elements);
}

pub fn clauses_query_generator<'e>(
    placeholder_offset: u8,
    clauses: &'e ClausesT,
) -> (String, Vec<&'e (dyn ToSql + Sync)>)
{
    let mut values_: Vec<&(dyn ToSql + Sync)> = vec![];
    let mut clauses_: String = "".to_string();
    let _val_arr: Vec<String>;

    if clauses.len() > 0 {
        let mut clauses_list: Vec<String> = vec![];

        let mut placeholder_index = placeholder_offset;
        for a_clause_tuple in clauses
        {
            placeholder_index += 1;
            let key = a_clause_tuple.m_field_name;
            if db_model::S_SINGLE_OPERANDS.contains(&a_clause_tuple.m_clause_operand)
            {
                clauses_list.push(" (".to_owned() + &key + &" ".to_owned() + &a_clause_tuple.m_clause_operand + &format!(" ${placeholder_index} ) "));
                values_.push(&a_clause_tuple.m_field_single_str_value as &(dyn ToSql + Sync));
            } else if (a_clause_tuple.m_clause_operand == "IN") || (a_clause_tuple.m_clause_operand == "NOT IN")
            {
                if a_clause_tuple.m_field_multi_values.len() == 0
                {
                    dlog(
                        &format!("Maleformed clauses 'IN/NOT IN': {:?}", dump_clauses(&clauses)),
                        constants::Modules::Sql,
                        constants::SecLevel::Fatal);
                    panic!("{}", format!("Maleformed clauses 'IN/NOT IN': {:?}", dump_clauses(&clauses)));
                }

                let mut tmp_placeholders: Vec<String> = vec![];
                for i in 0..a_clause_tuple.m_field_multi_values.len()
                {
                    tmp_placeholders.push(format!("${}", placeholder_index));
                    values_.push(&a_clause_tuple.m_field_multi_values[i] as &(dyn ToSql + Sync));
                    placeholder_index += 1;
                }
                placeholder_index -= 1;
                clauses_list.push(" (".to_owned() + &key + &" ".to_owned() + &a_clause_tuple.m_clause_operand + &" (".to_owned() + &tmp_placeholders.join(", ") + ")) ");
            } else if a_clause_tuple.m_clause_operand == "LIKE:OR"
            {
                panic!("LIKE:OR NOT IMPLEMENTED YET!");
            }
        }


        if (clauses_list.len() > 0) && (values_.len() > 0) {
            clauses_ = clauses_list.join(" AND ");
        }
    }

    return (clauses_, values_);
}
/*

*/

pub fn pre_query_generator<'e>(
    placeholder_offset: u8,
    clauses_: &'e ClausesT,
    order_: OrderT,
    limit_: LimitT) -> QueryElements<'e>
{
    let (mut concatenated_clauses, values_) = clauses_query_generator(placeholder_offset, clauses_);
    if concatenated_clauses.len() > 0 {
        concatenated_clauses = " WHERE ".to_owned() + &concatenated_clauses;
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

    let q_elements: QueryElements = QueryElements {
        m_clauses: concatenated_clauses,
        m_params: values_,
        m_order: order,
        m_limit: limit,
        m_complete_query: "".to_string(),
    };
    q_elements
}

pub fn q_customQuery(
    complete_query: &String,
    params: &Vec<&str>,
    do_log: bool) -> (bool, QVDRecordsT)
{
    if do_log {
        dlog(
            &complete_query.to_string(),
            constants::Modules::Sql,
            constants::SecLevel::Trace);
    }

    let mut out_rows: QVDRecordsT = vec![];
    let params_: Vec<_> = params.iter().map(|x| x as &(dyn ToSql + Sync)).collect();

    return match dbhandler().m_db.query(
        complete_query,
        &params_[..]) {
        Ok(rows) => {
            if do_log {
                dlog(
                    &format!("Query executed successfully: "),
                    constants::Modules::Sql,
                    constants::SecLevel::Trace);
            }

            if rows.len() == 0
            {
                // let _res: Vec<Row> = vec![];
                return (true, out_rows);
            }

            let mut res_cols_info: Vec<(String, String)> = vec![];
            for a_col in &*rows[0].columns() {
                res_cols_info.push((a_col.name().to_string(), a_col.type_().to_string()));
            }

            for a_row in &rows {
                // println!(">>> a_row: {:?}", a_row);
                let mut a_row_dict: HashMap<String, String> = HashMap::new();
                for col_inx in 0..a_row.len() {
                    let (col_name, col_type) = &res_cols_info[col_inx];
                    // let col_value: String = Row::get(a_row, col_inx);
                    let the_col_value: String = match &*col_type.clone() {
                        ("real" | "double precision") => {
                            let col_value: f64 = Row::get(a_row, col_inx);
                            convert_float_to_string(col_value, 11)
                        }
                        ("smallint" | "smallserial" | "int" | "serial" | "oid" | "bigint" | "bigserial" | "int4" | "int8") => {
                            let col_value: i64 = Row::get(a_row, col_inx);
                            col_value.to_string()
                        }
                        "numeric" => {
                            let col_value: i64 = Row::get(a_row, col_inx);
                            println!("failed on casting psql numeric to Rust i64!!!{:?}", col_value);
                            col_value.to_string()
                        }
                        "varchar" => {
                            let col_value: Option<String> = Row::get(a_row, col_inx);
                            if col_value.is_some() {
                                col_value.unwrap().to_string()
                            } else {
                                "".to_string()
                            }
                        }
                        ("text") => {
                            let col_value: String = Row::get::<_, String>(a_row, col_inx);
                            col_value
                        }
                        ("bool") => {
                            let col_value: bool = Row::get(a_row, col_inx);
                            col_value.to_string()
                        }
                        (_) => {
                            println!("UUUUU Unsetted col type {} {} ", col_type.clone(), col_name.clone());
                            let col_value: String = Row::get(a_row, col_inx);
                            println!("UUUUU Unsetted col type {} {} {}", col_type.clone(), col_name.clone(), col_value.clone());
                            col_value.to_string()
                        }
                    };
                    a_row_dict.insert(col_name.clone(), the_col_value);
                }
                out_rows.push(a_row_dict);
            }
            println!(">>> out_dict: {:?}", out_rows);
            (true, out_rows)
        }
        Err(e) => {
            dlog(
                &format!("Failed in cQ e: {:?} ", e),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            dlog(
                &format!("Failed in cQ query: {} ", complete_query),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            dlog(
                &format!("Failed in cQ params: [{:?}] ", params),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            (true, vec![])
        }
    };

    /*
        let mut    bound_list_for_log:Vec<String>=vec![];
            exec_res;
        if (to_bind_values.len() > 0)
        {
            query -> prepare(complete_query);
            for (String aKey : to_bind_values.keys())
            query->addBindValue(to_bind_values[aKey]);

            QMap < String, QVariant > sqlIterator(query->boundValues());
            for (auto i = sqlIterator.begin(); i != sqlIterator.end(); + + i)
            {
                bound_list_for_log.push(i.key().toUtf8() + ": " + i.value().to_string().toUtf8());
            }

            exec_res = query -> exec();
        } else {
            exec_res = query -> exec(complete_query);
        }


        if (do_log)
        CLog::log("Query Values: [" + bound_list_for_log.join(", ") + "]", "sql", "trace");

        QVDRecordsT
        records;
        if (!exec_res)
        {
            String
            err_text = query -> lastError().databaseText();
            if (uniqueConstraintFailed(err_text))
            return { true, records };

            CLog::log(err_text, "sql", "fatal");
            CLog::log("PSQL failed query1: " + complete_query, "sql", "fatal");
            CLog::log("PSQL failed values1: " + cutils::dumpIt(to_bind_values), "sql", "fatal");
            String
            executed_query = query -> executedQuery();
            CLog::log("PSQL failed executed_query: " + executed_query, "sql", "fatal");
            CLog::log("PSQL failed clauses2: " + err_text, "sql", "fatal");
            return { false, records };
        }

        while (query -> next())
        {
            QVDicT
            a_row;
            if (fields.len() > 0)
            {
                for (String a_filed: fields)
                {
                    a_row[a_filed] = query -> value(a_filed);
                }
            } else {
                // return results by position
                for (int i=0; i < field_count; i+ +)
                a_row[String::number(i)] = query->value(i);
            }
            records.push(a_row);

            QMap < String, QVariant > sqlIterator(query->boundValues());
    //    StringList bound_list_for_log;
    //    for (auto i = sqlIterator.begin(); i != sqlIterator.end(); ++i)
    //      bound_list_for_log.push(i.key().toUtf8() + ": " + i.value().to_string().toUtf8());
        }

        if (query -> isSelect())
        {
            if (do_log)
            CLog::log("Results: [" + format!(records.len()) + " rows] returned", "sql", "trace");
        }
        else if (query -> numRowsAffected() > 0)
        {
            CLog::log("Affected: [" + format!(query->numRowsAffected()) + " rows] afected", "sql", "trace");
        }

        query -> finish();
        delete
        query;

        return { true, records };
        */
}

pub fn exec_query(
    query_elements: &QueryElements,
    do_log: bool) -> (bool, QVDRecordsT)
{
    if do_log {
        dlog(
            &query_elements.m_complete_query.to_string(),
            constants::Modules::Sql,
            constants::SecLevel::Trace);
    }

    if do_log {
        dlog(
            &format!("Query Values: {:?} ", &query_elements.m_params),
            constants::Modules::Sql,
            constants::SecLevel::Trace);
    }
    let mut out_rows: QVDRecordsT = vec![];
    // let params: Vec<_> = query_elements.m_params.iter().map(|x| x as &(dyn ToSql + Sync)).collect();
    return match dbhandler().m_db.query(
        &query_elements.m_complete_query,
        &query_elements.m_params[..]) {
        Ok(rows) => {
            if do_log {
                dlog(
                    &format!("Query executed successfully: "),
                    constants::Modules::Sql,
                    constants::SecLevel::Trace);
            }

            if rows.len() == 0
            {
                // let res: Vec<Row> = vec![];
                return (true, out_rows);
            }

            // let rr = Row::columns(&rows[0]);
            let mut res_cols_info: Vec<(String, String)> = vec![];
            for a_col in &*rows[0].columns() {
                res_cols_info.push((a_col.name().to_string(), a_col.type_().to_string()));
            }

            for a_row in &rows {
                let mut a_row_dict: HashMap<String, String> = HashMap::new();
                for col_inx in 0..a_row.len() {
                    let (col_name, col_type) = &res_cols_info[col_inx];
                    let the_col_value: String = match &*col_type.clone() {
                        ("real" | "double precision" | "float8") => {
                            let col_value: f64 = Row::get(a_row, col_inx);
                            convert_float_to_string(col_value, 11)
                        }
                        ("int4") => {
                            let col_value: i32 = Row::get(a_row, col_inx);
                            col_value.to_string()
                        }
                        ("smallint" | "smallserial" | "int" | "serial" | "oid" | "bigint" | "bigserial" | "int4" | "int8") => {
                            let col_value: i64 = Row::get(a_row, col_inx);
                            col_value.to_string()
                        }
                        "numeric" => {
                            let col_value: i64 = Row::get(a_row, col_inx);
                            println!("failed2 on casting psql numeric to Rust i64!!!{:?}", col_value);
                            col_value.to_string()
                        }
                        "varchar" => {
                            let col_value: Option<String> = Row::get(a_row, col_inx);
                            if col_value.is_some() {
                                col_value.unwrap().to_string()
                            } else {
                                "".to_string()
                            }
                        }
                        "text" => {
                            let col_value: String = Row::get::<_, String>(a_row, col_inx);
                            col_value
                        }
                        "bool" => {
                            let col_value: bool = Row::get(a_row, col_inx);
                            col_value.to_string()
                        }
                        _ => {
                            println!("UUUUU2 Unsetted col type {} {} ", col_type.clone(), col_name.clone());
                            let col_value: String = Row::get(a_row, col_inx);
                            println!("UUUUU2 Unsetted col type {} {} {}", col_type.clone(), col_name.clone(), col_value.clone());
                            col_value.to_string()
                        }
                    };
                    a_row_dict.insert(col_name.clone(), the_col_value);

                    // let col_value: String = Row::get(a_row, col_inx);
                    // a_row_dict.insert(col_name.clone(), col_value);
                }
                out_rows.push(a_row_dict);
            }
            //println!(">>> out_dict: {:?}", out_rows);
            (true, out_rows)
        }
        Err(e) => {
            dlog(
                &format!("Failed in Q e: {:?} ", e),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            dlog(
                &format!("Failed in Q query: {} ", &query_elements.m_complete_query),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            dlog(
                &format!("Failed in Q params: [{:?}] ", &query_elements.m_params),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            (true, vec![])
        }
    };
}

/*

bool DbModel::handleMaybeDbLock(QSqlQuery& query)
{
  String dummeyLockErrMsg = "database is locked";
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
      CLog::log("handle Maybe Db Lock failed Thread: " + String::number((quint64)QThread::currentThread(), 16), "sql", "fatal");
      CLog::log("handle Maybe Db Lock" + query.lastError().databaseText(), "sql", "fatal");
      String executed_query = query.executedQuery();
      CLog::log("SQL failed executed_query: " + executed_query, "sql", "fatal");
      String err_text = query.lastError().databaseText();
      CLog::log("SQL failed clauses2: " + err_text, "sql", "fatal");
      if (err_text != dummeyLockErrMsg)
        return false;
    }
  }

  CLog::log("even handle Maybe DbLock didn't work!", "app", "fatal");
  return false;
}


const String UNIQUE_ERR_MSG_PREFIX1 = "UNIQUE constraint failed";
const String UNIQUE_ERR_MSG_PREFIX2 = "ERROR:  duplicate key value violates unique constraint";

bool DbModel::uniqueConstraintFailed(const String& msg)
{
  if (msg.midRef(0, UNIQUE_ERR_MSG_PREFIX1.len()).to_string() == UNIQUE_ERR_MSG_PREFIX1)
    return true;
  if (msg.midRef(0, UNIQUE_ERR_MSG_PREFIX2.len()).to_string() == UNIQUE_ERR_MSG_PREFIX2)
    return true;
  return false;
}

*/
pub fn q_insert(
    table: &str,
    values: &HashMap<&str, &(dyn ToSql + Sync)>,
    do_log: bool) -> bool
{
    return insert_to_psql(table, values, do_log);
}


pub fn insert_to_psql(
    table: &str,
    values: &HashMap<&str, &(dyn ToSql + Sync)>,
    do_log: bool) -> bool
{
    let mut position: u8 = 0;
    let mut placeholders: Vec<String> = vec![];
    let mut the_keys: Vec<&str> = vec![];
    let mut params: Vec<&(dyn ToSql + Sync)> = vec![];
    for (k, v) in values {
        position += 1;
        let pos = format!("${}", position);
        placeholders.push(pos);
        the_keys.push(k);
        params.push(*v);
    }
    // let keys = values.iter().map(|(&k, &v)| k).collect::<Vec<_>>(); //values.keys();
    let placeholders = placeholders.join(", ");
    let the_keys = the_keys.join(", ");
    let complete_query: String = "INSERT INTO ".to_owned() + table + " (" + &*the_keys + ") VALUES (" + &*placeholders + "); "; //BEGIN; COMMIT;

    if do_log {
        dlog(
            &format!("{}", complete_query),
            constants::Modules::Sql,
            constants::SecLevel::Info);
    }

    let _log_values: Vec<&str> = vec![];

    // let params: Vec<_> = params.iter().map(|x| x as &(dyn ToSql + Sync)).collect();
    return match dbhandler().m_db.execute(&complete_query, &params) {
        Ok(_v) => { true }
        Err(e) => {
            dlog(
                &format!("failed in insert {:?}", e),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            dlog(
                &format!("Query in PSQL: {:?}", complete_query),
                constants::Modules::Sql,
                constants::SecLevel::Error);
            dlog(
                &format!("Params in PSQL: {:?}", params),
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
    values: &HashMap<&str, &(dyn ToSql + Sync)>,
    do_log: bool) -> bool
{
    let only_clause = simple_eq_clause(controlled_field, controlled_value);
    let clauses: ClausesT = vec![only_clause];

    // controll if the record already existed
    let (status, records) = q_select(
        table,
        &vec![controlled_field],     // fields
        clauses.clone(), //clauses
        vec![],   // order
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
            clauses,  // update clauses
            do_log);

        return true;
    }

    // insert new entry
    let mut insert_values = values.clone();
    insert_values.insert(controlled_field, &controlled_value as &(dyn ToSql + Sync));
    return q_insert(
        table,     // table
        &insert_values, // values to insert
        true,
    );
}

pub fn prepare_to_update<'e>(
    table: &'e str,
    upd_values: &'e HashMap<&str, &(dyn ToSql + Sync)>,
    clauses: &'e ClausesT) -> QueryElements<'e>
{
    let ord_ = vec![];
    let mut query_elements: QueryElements = pre_query_generator(upd_values.len() as u8, clauses, ord_, 0);

    let mut updates: Vec<String> = vec![];
    let mut position = 0;
    let mut finall_upd_values: Vec<&(dyn ToSql + Sync)> = vec![]; //query_elements.m_params
    for (a_key, &a_value) in upd_values
    {
        position += 1;
        let the_key = a_key.clone();
        let the_single_update = (the_key.to_owned() + &format!("= ${}", position)).clone();
        updates.push(the_single_update);
        finall_upd_values.push(a_value);
    }
    for &a_value in &query_elements.m_params {
        finall_upd_values.push(a_value);
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
    update_values: &HashMap<&str, &(dyn ToSql + Sync)>,
    update_clauses: ClausesT,
    do_log: bool) -> bool
{
    let query_elements = prepare_to_update(table, update_values, &update_clauses);
    let _upd_res = exec_query(
        &query_elements,
        do_log);

    return true;
}
/*


    bool DbModel::clauseContains(const ClausesT& clauses, const String& filed_name)   //legacy queryHas
    {
      for (ModelClause a_clause : clauses)
      {
        if (a_clause.m_field_name == filed_name)
          return true;
       }
       return false;
    }










     */