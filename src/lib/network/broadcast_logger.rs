use std::collections::HashMap;
use postgres::types::{ToSql};
use crate::{application, constants, cutils, dlog, machine};
use crate::lib::custom_types::{CDateT, QVDRecordsT};
use crate::lib::database::abs_psql::{ModelClause, q_insert, q_select};
use crate::lib::database::tables::C_LOGS_BROADCAST;

//old_name_was listSentBlocks
pub fn list_sent_blocks(after_that_: &CDateT, fields: Vec<&str>) -> QVDRecordsT
{
    let mut after_that = after_that_.clone();
    if after_that == ""
    {
        let now_ = application().get_now();
        if machine().is_in_sync_process(false) {
            let back_in_time = application().get_cycle_by_minutes()  / 40;
            after_that = application().minutes_before(back_in_time, &now_);
        } else {
            let back_in_time = application().get_cycle_by_minutes()  / 20;
            after_that = application().minutes_before(back_in_time, &now_);
        }
    }
    let (status, records) = q_select(
        C_LOGS_BROADCAST,
        fields,
        vec![ModelClause {
            m_field_name: "lb_send_date",
            m_field_single_str_value: &after_that as &(dyn ToSql + Sync),
            m_clause_operand: ">=",
            m_field_multi_values: vec![],
        }],
        vec![],
        0,
        true);
    if !status {
        return vec![];
    }

    records
}

//old_name_was listSentBloksIds
pub fn list_sent_bloks_ids() -> Vec<String>
{
    let rows: QVDRecordsT = list_sent_blocks(&"".to_string(), vec!["lb_type", "lb_code", "lb_sender", "lb_receiver"]);
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

//old_name_was addSentBlock
pub fn add_sent_block(values: &mut HashMap<&str, &(dyn ToSql + Sync)>) -> bool
{
    dlog(
        &format!("add SentBlock: {:?}", &values),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    q_insert(
        C_LOGS_BROADCAST,
        values,
        false);
    return true;
}

