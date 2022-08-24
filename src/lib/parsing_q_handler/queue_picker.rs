use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, ccrypto, constants, cutils, dlog, machine};
use crate::lib::block_utils::unwrap_safed_content_for_db;
use crate::lib::custom_types::QVDicT;
use crate::lib::database::abs_psql::{q_custom_query, q_update, simple_eq_clause};
use crate::lib::database::tables::{C_PARSING_Q, C_PARSING_Q_FIELDS};

//old_name_was smartPullQ
pub fn smart_pull_q() -> bool
{
    let (complete_query, values) = prepare_smart_query(1);

    let no_prerequisites = ",".to_string();
    let params = vec![&no_prerequisites as &(dyn ToSql + Sync)];
    let (status, records) = q_custom_query(
        &complete_query,
        &params,
        false);

    if records.len() == 0
    {
        dlog(
            &format!("No processable packet in parsing Q!"),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return true;
    }

    let packet = &records[0];
    let (status, sf_version, unwrap_res) = unwrap_safed_content_for_db(&packet["pq_payload"].to_string());
    if !status
    {
        // purge record
        // reputation report
        return false;
    }
    /*
    let (status, Jpayload) = cutils::controlled_str_to_json(&unwrap_res);
    packet["pq_payload"] = Jpayload;

    increase_to_parse_attempts_count(packet);

      let(status, should_purge_record) = handlePulledPacket(packet);
      if !should_purge_record
      {
          CLog::log("Why not purge1! pq_type(" + packet["pq_type"].to_string() + ") block(" + cutils::hash8c(packet["pq_code"].to_string()) + ")" + " from(" + packet["pq_sender"].to_string() + ")", "app", "error");

        } else {
          DbModel::dDelete(
            C_PARSING_Q,
              {
                  {"pq_sender", packet["pq_sender"]},
                  {"pq_type", packet["pq_type"]},
                  {"pq_code", packet["pq_code"]}
                });
        }
*/
    /*
      return status;
      */
    true
}


pub fn prepare_smart_query(limit: u16) -> (String, QVDicT)
{
    let fields = C_PARSING_Q_FIELDS.iter().map(|&x| x).collect::<Vec<&str>>().join(", ").to_string();
    let limit = format!(" LIMIT {} ", limit);

    // TODO: make a more intelligence query
    let mut query: String;
    let query_number = ccrypto::get_random_number(100);
    // uint8_t  = rand() % 100;
    if machine().is_in_sync_process(false)
    {
        dlog(
            &format!("Random query number: {}", query_number),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        if query_number < 60
        {
            // since it is in sync phase, so maybe better order is based on creationdate
            // TODO: optimize it to prevent cheater to vector attack
            query = "SELECT ".to_owned() + &fields + " FROM " + C_PARSING_Q;
            query += &(" WHERE pq_prerequisites=$1 ORDER BY pq_connection_type ASC, pq_creation_date ASC ".to_owned() + &limit);
        } else if (query_number > 60) && (query_number < 90)
        {
            query = "SELECT ".to_owned() + &fields + " FROM " + C_PARSING_Q;
            query += &(" WHERE pq_prerequisites=$1 ORDER BY pq_connection_type ASC, pq_parse_attempts ASC, pq_receive_date ASC ".to_owned() + &limit);
        } else {
            query = "SELECT ".to_owned() + &fields + " FROM " + C_PARSING_Q;
            query += &(" WHERE pq_prerequisites=$1 ".to_owned() + &limit);
        }
    } else {
        if query_number < 60
        {
            query = "SELECT ".to_owned() + &fields + " FROM " + C_PARSING_Q;
            query += &(" WHERE pq_prerequisites=$1 ORDER BY pq_connection_type ASC, pq_parse_attempts ASC, pq_receive_date ASC ".to_owned() + &limit);
        } else if (query_number > 60) && (query_number < 90)
        {
            query = "SELECT ".to_owned() + &fields + " FROM " + C_PARSING_Q;
            query += &(" WHERE pq_prerequisites=$1 ORDER BY pq_connection_type ASC, pq_creation_date ASC ".to_owned() + &limit);
        } else {
            query = "SELECT ".to_owned() + &fields + " FROM " + C_PARSING_Q;
            query += &(" WHERE pq_prerequisites=$1 ".to_owned() + &limit);
        }
    }

    let values: QVDicT = HashMap::from([
        ("pq_prerequisites".to_string(), ",".to_string())
    ]);
    return (query, values);
}

//old_name_was increaseToparseAttempsCountSync
pub fn increase_to_parse_attempts_count(packet: &QVDicT) -> bool
{
    let mut parse_attempts = packet["pq_parse_attempts"].parse::<i64>().unwrap_or(0);
    parse_attempts = parse_attempts + 1;
    let now_ = application().get_now();
    let update_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("pq_parse_attempts", &parse_attempts as &(dyn ToSql + Sync)),
        ("pq_last_modified", &now_ as &(dyn ToSql + Sync))
    ]);
    return q_update(
        C_PARSING_Q,
        &update_values,
        vec![
            simple_eq_clause("pq_type", &packet["pq_type"].to_string()),
            simple_eq_clause("pq_code", &packet["pq_code"].to_string()),
            simple_eq_clause("pq_sender", &packet["pq_sender"].to_string()),
        ],
        false);
}
