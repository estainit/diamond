use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, dlog};
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{ClausesT, CMPAIValueT, LimitT, OrderT, QVDRecordsT};
use crate::lib::database::abs_psql::{q_delete, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::{C_MACHINE_BLOCK_BUFFER};

//old_name_was searchBufferedDocs
pub fn search_buffered_docs(
    clauses: ClausesT,
    fields: Vec<&str>,
    order: OrderT,
    limit: LimitT) -> QVDRecordsT
{
    let (_status, records) = q_select(
        C_MACHINE_BLOCK_BUFFER,
        fields,
        clauses,
        order,
        limit,
        false);
    return records;
}

// old name was removeFromBuffer
pub fn remove_from_buffer(clauses: ClausesT) -> bool
{
    q_delete(
        C_MACHINE_BLOCK_BUFFER,
        clauses,
        false);

    return true;
}

// js name was pushToBlockBuffer
pub fn push_to_block_buffer(
    doc: &Document,
    dp_cost: CMPAIValueT,
    mp_code: &String) -> (bool, String)
{
    let msg: String;
    //listener.doCallAsync('APSH_before_push_doc_to_buffer_async', args);

    let dbl_chk = search_buffered_docs(
        vec![
            simple_eq_clause("bd_mp_code", mp_code),
            simple_eq_clause("bd_doc_hash", &doc.get_doc_hash()),
        ],
        vec!["bd_doc_hash"],
        vec![],
        0,
    );
    if dbl_chk.len() > 0
    {
        msg = format!("Tried to insert in buffer duplicated {}", doc.get_doc_identifier());
        dlog(
            &msg,
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return (false, msg);
    }

    let payload: String = doc.safe_stringify_doc(true);

    let doc_hash = doc.get_doc_hash();
    let doc_type = doc.get_doc_type();
    let doc_class = doc.get_doc_class();
    let dp_cost_i64 = dp_cost as i64;
    let payload_len_i32 = payload.len() as i32;
    let now_ = application().now();
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("bd_mp_code", &mp_code as &(dyn ToSql + Sync)),
        ("bd_insert_date", &now_ as &(dyn ToSql + Sync)),
        ("bd_doc_hash", &doc_hash as &(dyn ToSql + Sync)),
        ("bd_doc_type", &doc_type as &(dyn ToSql + Sync)),
        ("bd_doc_class", &doc_class as &(dyn ToSql + Sync)),
        ("bd_payload", &payload as &(dyn ToSql + Sync)),
        ("bd_dp_cost", &dp_cost_i64 as &(dyn ToSql + Sync)),
        ("bd_doc_len", &payload_len_i32 as &(dyn ToSql + Sync)),
    ]);
    let status: bool = q_insert(
        C_MACHINE_BLOCK_BUFFER,
        &values,
        true);
    dlog(
        &format!("Insert a document in block buffer, values: {:#?}", values),
        constants::Modules::App,
        constants::SecLevel::Info);
    if status
    {
        return (
            true,
            format!("The document have been pushed into buffer. {}", doc.get_doc_identifier())
        );
    }

    dlog(
        &format!("Failed in push doc to block buffer, values: {:#?}", values),
        constants::Modules::App,
        constants::SecLevel::Error);
    return (false, "Failed in push doc to block buffer".to_string());
}
