use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, cutils, dlog};
use crate::lib::custom_types::{ClausesT, LimitT, OrderT, QVDRecordsT, VString};
use crate::lib::database::abs_psql::{q_custom_query, q_select, q_update, simple_eq_clause};
use crate::lib::database::tables::C_PARSING_Q;


// appends given Prerequisites for given block
//old_name_was appendPrerequisites
pub fn append_prerequisites(
    block_hash: &String,
    prerequisites: &VString,
    pq_type: &String) -> bool
{
    let block_identifier = format!("{}", cutils::hash8c(&block_hash));
    if prerequisites.len() == 0
    { return true; }

    let mut clauses: ClausesT = vec![simple_eq_clause("pq_code", block_hash)];
    if pq_type != ""
    {
        clauses.push(simple_eq_clause("pq_type", pq_type));
    }
    let records = search_parsing_q(
        clauses,
        vec!["pq_type", "pq_code", "pq_prerequisites"],
        vec![],
        0);

    if records.len() == 0
    {
        dlog(
            &format!("Wrong request to append requisites to a {} which does not exist in parsing q!",
                     block_identifier),
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return false;
    }

    let mut current_prereq = cutils::convert_comma_separated_string_to_string_vector(
        &records[0]["pq_prerequisites"].to_string());

    dlog(
        &format!(
            "in {} adding new prerequisites({:?}) to existed prerequisites({:?})",
            block_identifier, prerequisites, current_prereq),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    current_prereq = cutils::array_add(&current_prereq, &prerequisites);
    dlog(
        &format!(
            "{} final1 prerequisities({:?})",
            block_identifier, current_prereq),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);
    current_prereq.sort();
    dlog(
        &format!(
            "{} final2 prerequisities: {:?}",
            block_identifier, current_prereq),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    let prerequisites = format!(",{}", current_prereq.join(","));
    let last_modified = application().get_now();
    let update_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("pq_prerequisites", &prerequisites as &(dyn ToSql + Sync)),
        ("pq_last_modified", &last_modified as &(dyn ToSql + Sync))
    ]);

    return q_update(
        C_PARSING_Q,
        &update_values,
        vec![simple_eq_clause("pq_code", block_hash)],
        false,
    );
}

//old_name_was searchParsingQ
pub fn search_parsing_q(
    clauses: ClausesT,
    fields: Vec<&str>,
    order: OrderT,
    limit: LimitT) -> QVDRecordsT
{
    let (_status, records) = q_select(
        C_PARSING_Q,
        fields,
        clauses,
        order,
        limit,
        true,
    );
    // let fields_str: String = fields_array.join(", ");
    // let qElms: QueryElements = pre_query_generator(0, clauses, order, limit);
    // let (_status, records) = q_customQuery(
    //     &("SELECT ".to_owned() + &fields_str + " FROM " + C_PARSING_Q + &qElms.m_clauses + &qElms.m_order + &qElms.m_limit),
    //     &qElms.m_params,
    //     true);
    return records;
}

//old_name_was removePrerequisites
pub fn remove_prerequisites(block_hash: &String)
{
    let q = format!("SELECT pq_type, pq_code, pq_prerequisites FROM {} WHERE pq_prerequisites LIKE %{}%",
                    C_PARSING_Q, block_hash);

    let (_status, records) = q_custom_query(
        &q,
        &vec![],
        false);

    if records.len() == 0
    { return; }

    for a_block in records
    {
        let mut prerequisites = a_block["pq_prerequisites"].to_string().replace(block_hash, "");
        prerequisites = cutils::normalize_comma_seperated_string(&prerequisites);

        let update_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("pq_prerequisites", &prerequisites as &(dyn ToSql + Sync))
        ]);

        q_update(
            C_PARSING_Q,
            &update_values,
            vec![
                simple_eq_clause("pq_type", &a_block["pq_type"]),
                simple_eq_clause("pq_code", &a_block["pq_code"]),
            ],
            false);
    };
}

