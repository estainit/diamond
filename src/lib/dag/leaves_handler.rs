use std::collections::HashMap;
use postgres::types::ToSql;
use serde::{Serialize, Deserialize};
use crate::{application, constants, cutils, dlog};
use crate::lib::custom_types::{CBlockHashT, CDateT, TimeByMinutesT};
use crate::lib::database::abs_psql::q_upsert;
use crate::lib::database::tables::C_KVALUE;
use crate::lib::k_v_handler::get_value;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LeaveBlock {
    pub m_block_type: String,
    pub m_block_hash: String,
    pub m_creation_date: String,
}

pub fn remove_from_leave_blocks(leaves: &Vec<String>) -> (bool, String)
{
    let current_leaves: HashMap<String, LeaveBlock> = get_leave_blocks(&"".to_string());
    let mut new_leaves: HashMap<String, LeaveBlock> = HashMap::new();

    for (a_key, a_leave) in current_leaves
    {
        if !leaves.contains(&a_key)
        {
            // push it to new vector
            new_leaves.insert(a_key, a_leave);
        }
    }
    let serialized_leaves: String = serde_json::to_string(&new_leaves).unwrap();

    // update db
    let kv_last_modified = application().get_now();
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("kv_value", &serialized_leaves as &(dyn ToSql + Sync)),
        ("kv_last_modified", &kv_last_modified as &(dyn ToSql + Sync)),
    ]);

    q_upsert(
        C_KVALUE,
        "kv_key",
        "dag_leave_blocks",
        &values,
        true);

    return (true, "".to_string());
}

//old_name_was getLeaveBlocks
pub fn get_leave_blocks(only_before_date: &CDateT) -> HashMap<String, LeaveBlock>
{
    let value: String = get_value("dag_leave_blocks");
    if value == "" {
        return HashMap::new();
    }

    let deser_leaves: HashMap<String, LeaveBlock> = serde_json::from_str(&value).unwrap();

    if *only_before_date == ""
    {
        return deser_leaves;
    }

    // filter older leaves  FIXME: complete it
    let mut filtered_leaves: HashMap<String, LeaveBlock> = HashMap::new();
    for (a_key, a_leave) in deser_leaves
    {
        if (a_leave.m_block_type == constants::block_types::GENESIS) ||
            (a_leave.m_creation_date < *only_before_date)
        {
            filtered_leaves.insert(a_key, a_leave);
        }
    }

    return filtered_leaves;
}

pub fn add_to_leave_blocks(
    block_hash: &CBlockHashT,
    creation_date: &CDateT,
    block_type: &String) -> (bool, String)
{
    let mut current_leaves: HashMap<String, LeaveBlock> = get_leave_blocks(&"".to_string());
    let a_leave: LeaveBlock = LeaveBlock {
        m_block_type: block_type.clone(),
        m_block_hash: block_hash.to_string(),
        m_creation_date: creation_date.clone(),
    };
    current_leaves.insert(block_hash.to_string(), a_leave);

    let kv_value = serde_json::to_string(&current_leaves).unwrap();
    let kv_last_modified = application().get_now();
    let update_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("kv_value", &kv_value as &(dyn ToSql + Sync)),
        ("kv_last_modified", &kv_last_modified as &(dyn ToSql + Sync)),
    ]);

    q_upsert(
        C_KVALUE,
        "kv_key",
        "dag_leave_blocks",
        &update_values,
        true);

    return (true, "".to_string());
}

//old_name_was getFreshLeaves
pub fn get_fresh_leaves() -> HashMap<String, LeaveBlock>
{
    // the leaves younger than two cylce (24 hours) old
    let leaves: HashMap<String, LeaveBlock> = get_leave_blocks(&"".to_string());

    println!("mmmmmmm get_fresh_leaves {:?}", leaves);

    dlog(
        &format!("current leaves: {}", serde_json::to_string(&leaves).unwrap()),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    if leaves.keys().len() == 0 {
        return leaves;
    }
    println!("mmmmmmm get_fresh_leaves oooooooooooo", );

    let mut refreshes: HashMap<String, LeaveBlock> = HashMap::new();
    let now_ = application().get_now();
    for (a_key, a_leave) in leaves {
        let leave_age: TimeByMinutesT = application().time_diff(
            a_leave.m_creation_date.clone(),
            now_.clone()).as_minutes;

        dlog(
            &format!(
                "The leave creation date: {}, now: {}",
                a_leave.m_creation_date.clone(), now_.clone()),
            constants::Modules::App,
            constants::SecLevel::Debug);

        let mut msg: String = format!(
            "The leave(#{}) age ({}) minutes is ",
            cutils::hash6c(&a_key), leave_age);
        if leave_age < application().get_cycle_by_minutes() * 2 {
            msg += " younger ";
        } else {
            msg += " older ";
        }
        msg += " than 2 cycles ";

        dlog(
            &msg,
            constants::Modules::App,
            constants::SecLevel::Info);

        if leave_age < application().get_cycle_by_minutes() * 2 {
            refreshes.insert(a_key, a_leave);
        }
    }

    return refreshes;
}


//old_name_was hasFreshLeaves
pub fn has_fresh_leaves() -> bool
{
    let keys = get_fresh_leaves().keys().cloned().collect::<Vec<String>>();
    println!("mmmmmmm fresh leaves keys {}, {:?}", keys.len(), keys);
    keys.len() > 0
}

