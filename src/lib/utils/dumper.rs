use std::collections::HashMap;
use crate::constants;
use crate::lib::custom_types::{ClausesT, QVDRecordsT};
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::TOutput;

pub fn dump_it<T>(x: T) -> String {
    println!("here in dumper");
    return "".to_string();
}

pub fn dump_clauses(clauses: &ClausesT) -> String {
    let mut out: String = "".to_string();
    for a_clause_tuple in clauses
    {
        let mvs = format!("{:?}", a_clause_tuple.m_field_multi_values);
        let the_outs: Vec<String> = vec![
            format!("m_field_name: {}", a_clause_tuple.m_field_name),
            format!("m_field_single_str_value: {:?}", a_clause_tuple.m_field_single_str_value),
            format!("m_clause_operand: {}", a_clause_tuple.m_clause_operand),
            format!("m_field_multi_values: {:?}", mvs)];

        out += &format!("{:?}", the_outs.join("    ")).clone();
    }
    return out.to_string();
}

pub fn dump_vec_of_str(s: &Vec<String>) -> String {
    let prefix_tabs = constants::TAB;

    let mut out: String = "".to_string();
    for a_vec in s {
        out += &(constants::NL.to_owned() + &prefix_tabs + &a_vec);
    }
    return out;
}

pub fn dump_vec_of_t_output(s: &Vec<TOutput>) -> String {
    let prefix_tabs = constants::TAB;

    let mut out: String = "".to_string();
    for an_out in s {
        out += &(constants::NL.to_owned() + &prefix_tabs + &an_out.dump());
    }
    return out;
}

pub fn dump_hashmap_of_str(s: &HashMap<&str, &str>) -> String {
    let prefix_tabs = constants::TAB;

    let mut out: String = "".to_string();
    for (k, v) in s {
        out += &(constants::NL.to_owned() + &prefix_tabs + &k + ": " + &v);
    }
    return out;
}

pub fn dump_hashmap_of_QVDRecordsT(s: &QVDRecordsT) -> String {
    let prefix_tabs = constants::TAB;

    let mut out: String = "".to_string();
    for a_hash_map in s {
        out += &(constants::NL.to_owned() + &prefix_tabs + &*dump_hashmap_of_string_string(a_hash_map));
    }
    return out;
}

pub fn dump_hashmap_of_string_string(s: &HashMap<String, String>) -> String {
    let prefix_tabs = constants::TAB;

    let mut out: String = "".to_string();
    for (k, v) in s {
        out += &(constants::NL.to_owned() + &prefix_tabs + &k + ": " + &v);
    }
    return out;
}

pub fn dump_hashmap_of_str_string(s: &HashMap<&str, String>) -> String {
    let prefix_tabs = constants::TAB;

    let mut out: String = "".to_string();
    for (k, v) in s {
        out += &(constants::NL.to_owned() + &prefix_tabs + &k + ": " + &v);
    }
    return out;
}

pub fn dump_hashmap_of_str_str(s: &HashMap<&str, &str>) -> String {
    let prefix_tabs = constants::TAB;

    let mut out: String = "".to_string();
    for (k, v) in s {
        out += &(constants::NL.to_owned() + &prefix_tabs + &k + ": " + &v);
    }
    return out;
}

pub fn dump_hashmap_of_string_f64(s: &HashMap<String, f64>) -> String {
    let prefix_tabs = constants::TAB;

    let mut out: String = "".to_string();
    for (k, v) in s {
        out += &(constants::NL.to_owned() + &prefix_tabs + &k + ": " + &*format!("{}", v));
    }
    return out;
}


