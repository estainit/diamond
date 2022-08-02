// use crate::lib::constants as CConsts;
// use crate::lib::custom_types::CDateT;


use crate::constants;
use crate::lib::custom_types::ClausesT;

pub fn dump_it<T>(x: T) -> String {
    println!("here in dumper");
    return "".to_string();
}

pub fn dump_clauses(clauses: &ClausesT) -> String {
    let mut out: String = "".to_string();
    for &a_clause_tuple in clauses
    {
        let mvs = format!("{:?}", a_clause_tuple.m_field_multi_values);
        let the_outs: Vec<String> = vec![
            format!("m_field_name: {}", a_clause_tuple.m_field_name),
            format!("m_field_single_str_value: {}", a_clause_tuple.m_field_single_str_value),
            format!("m_clause_operand: {}", a_clause_tuple.m_clause_operand),
            format!("m_field_multi_values: {:?}", mvs)];

        out += &format!("{:?}", the_outs.join("    ")).clone();
    }
    return out.to_string();
}

pub fn dump_vec_of_str(s: &Vec<String>) -> String {
    let prefix_tabs = "\t ";

    let mut out: String = "".to_string();
    for a_vec in s {
        out += &(constants::NL.to_owned() + &prefix_tabs + &a_vec);
    }
    return out;
}