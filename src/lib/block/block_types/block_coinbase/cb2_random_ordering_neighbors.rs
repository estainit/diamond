use std::collections::HashMap;
use crate::{application, ccrypto, cutils, machine};
use crate::lib::constants;
use crate::lib::custom_types::{CDateT, QSDicT, QVDRecordsT, TimeBySecT};
use crate::lib::dlog::dlog;
use crate::lib::machine::machine_neighbor::get_neighbors;
use crate::lib::services::dna::dna_handler::{get_machine_shares};
use crate::lib::utils::dumper::dump_hashmap_of_qvd_records;

//old_name_was haveIFirstHashedEmail
pub fn if_i_have_first_hashed_email(order: &str) -> bool
{
    let (cycle, machine_email, machine_key, emails_hash_dict) = make_email_hash_dictionary();
    let mut keys: Vec<String> = emails_hash_dict.keys().cloned().collect::<Vec<String>>();

    if order == "asc" {
        keys.sort();
    } else {
        // reverse it
        keys.sort();
        keys.reverse();
    }
    dlog(
        &format!("Ordered emails_hash_dict {:?}", keys),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    for i in 0..keys.len() {
        dlog(
            &format!("{}. candidate email for issueing CB {} {} ", i + 1, emails_hash_dict[&keys[i]], cutils::hash8c(&keys[i])),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
    }
    let machine_index: i32 = keys.iter().position(|r| r == &machine_key).unwrap() as i32; // keys.indexOf(machine_key)
    if machine_index == 0 {
        // the machin has minimum hash, so can generate the coinbase
        dlog(
            &format!("Machine has the lowest/heighest hash: {} {} ", machine_email, machine_key),
            constants::Modules::CB,
            constants::SecLevel::Info);
        return true;
    }

    // if the machine email hash is not the smalest,
    // control it if based on time passed from coinbase-cycle can create the coinbase?
    let now_ = application().get_now();
    let (
        _backer_address,
        _shares,
        mut percentage) = get_machine_shares(&now_);

    println!("kkkkkkkk 2");
    percentage = (percentage / 5.0) + 1.0;
    let mut sub_cycle = 12.0 + percentage;
    if application().cycle_length() != 1 {
        sub_cycle = 6.0 + percentage; // who has more shares should try more times to create a coinbase block
    }

    let now_ = application().get_now();
    let mut cb_email_counter = application().get_coinbase_age_by_seconds(&now_);
    println!("jjjjj sub_cycle: {}", sub_cycle);
    let sub2 =  application().get_cycle_by_seconds() / sub_cycle as TimeBySecT;
    println!("jjjjj kk: {}", sub2);
    cb_email_counter = cb_email_counter / sub2;
    println!("kkkkkkkk 2a {}", cb_email_counter);
    dlog(
        &format!("coinbase email counter cycle {} {} > {}", cb_email_counter, cb_email_counter, machine_index),
        constants::Modules::CB,
        constants::SecLevel::Info);
    println!("kkkkkkkk 3");
    if cb_email_counter > machine_index as TimeBySecT {
        // it is already passed time and if still no one create the block it is my turn to create it
        dlog(
            &format!("It already passed {} of 10 dividend of a cycle and now it's my turn {} to issue coinbase!", cb_email_counter, machine_email),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
        return true;
    }
    println!("kkkkkkkk 4");
    dlog(
        &format!("Machine has to wait To Create Coinbase Block! (if does not receive the fresh CBB) keys({}::{})", cycle, machine_email),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);
    return false;
}

//old_name_was makeEmailHashDict
pub fn make_email_hash_dictionary() -> (String, String, String, QSDicT)
{
    let mut emails_hash_dict: QSDicT = HashMap::new();
    let now_ = application().get_now();
    let cycle: CDateT = application().get_coinbase_cycle_stamp(&now_);
    let machine_email: String = machine().get_pub_email_info().m_address.clone();
    let machine_key: String = ccrypto::keccak256(&(cycle.clone() + "::" + &*machine_email));
    emails_hash_dict.insert(machine_key.clone(), machine_email.clone());

    let neightbors: QVDRecordsT = get_neighbors(
        "",
        constants::YES,
        "",
        0,
        "");
    dlog(
        &format!("neightbors in makeEmail Hash Dict: {}", dump_hashmap_of_qvd_records(&neightbors)),
        constants::Modules::CB,
        constants::SecLevel::Trace);

    for neighbor in neightbors
    {
        let key: String = ccrypto::keccak256(&(cycle.clone() + "::" + &neighbor["n_email"]));
        emails_hash_dict.insert(key, neighbor["n_email"].to_string());
    }
    return (cycle.to_string(), machine_email.clone(), machine_key.to_string(), emails_hash_dict);
}
