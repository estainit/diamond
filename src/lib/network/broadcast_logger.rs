use std::collections::HashMap;
use crate::{constants, cutils, dlog, machine};
use crate::lib::custom_types::{CDateT, QVDRecordsT};
use crate::lib::database::abs_psql::{ModelClause, q_insert, q_select};
use crate::lib::database::tables::STBL_LOGS_BROADCAST;
use crate::lib::utils::dumper::dump_it;


pub fn listSentBlocks(after_that_: &CDateT, fields: &Vec<&str>) -> QVDRecordsT
{
    let mut after_that = after_that_.clone();
    if after_that == ""
    {
        if machine().is_in_sync_process(false) {
            after_that = cutils::minutes_before(cutils::get_cycle_by_minutes() / 40, cutils::get_now());
        } else {
            after_that = cutils::minutes_before(cutils::get_cycle_by_minutes() / 20, cutils::get_now());
        }
    }
    let (status, records) = q_select(
        STBL_LOGS_BROADCAST,
        fields,
        &vec![&ModelClause {
            m_field_name: "lb_send_date",
            m_field_single_str_value: &*after_that,
            m_clause_operand: ">=",
            m_field_multi_values: vec![],
        }],
        &vec![],
        0,
        true);
    if !status {
        return vec![];
    }

    records
}

pub fn listSentBloksIds() -> Vec<String>
{
    let rows: QVDRecordsT = listSentBlocks(&"".to_string(), &vec!["lb_type", "lb_code", "lb_sender", "lb_receiver"]);
    let mut out: Vec<String> = vec![];
    for a_row in rows {
        out.push(vec![
            a_row["lb_type"].to_string(),
            a_row["lb_code"].to_string(),
            a_row["lb_sender"].to_string(),
            a_row["lb_receiver"].to_string(),
        ].join("").to_string()
        );
    }
    return out;
}


pub fn addSentBlock(values: &mut HashMap<&str, &str>) -> bool
{
    if values["lb_send_date"] == "" {
        values["lb_send_date"] = &*cutils::get_now();
    }

    dlog(
        &format!("add SentBlock: {}", dump_it(values)),
        constants::Modules::App,
        constants::SecLevel::Trace);

    q_insert(STBL_LOGS_BROADCAST, values, false);
    return true;
}

