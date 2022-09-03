use std::collections::HashMap;
use actix_web::{get, web};
use crate::application;
use crate::lib::custom_types::{QVDRecordsT, VString};
use crate::lib::dag::dag::search_in_dag;
use crate::lib::dag::leaves_handler::{get_fresh_leaves, get_leave_blocks, LeaveBlock};
use crate::lib::dag::missed_blocks_handler::list_missed_blocks;
use crate::lib::database::abs_psql::OrderModifier;
use crate::lib::file_handler::file_handler::list_exact_files;
use crate::lib::parsing_q_handler::queue_utils::search_parsing_q;
use crate::lib::sending_q_handler::sending_q_handler::fetch_from_sending_q;

#[get("/dagInfo")]
pub async fn dag_info() -> web::Json<QVDRecordsT>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let blocks = search_in_dag(
            vec![],
            vec!["b_hash", "b_type", "b_cycle", "b_confidence", "b_signals", "b_trxs_count",
                 "b_docs_count", "b_ancestors", "b_creation_date"],
            vec![
                &OrderModifier { m_field: "b_creation_date", m_order: "ASC" },
                &OrderModifier { m_field: "b_id", m_order: "ASC" },
                &OrderModifier { m_field: "b_hash", m_order: "ASC" },
            ],
            0,
            false);
        blocks
    }).await.expect("Failed in retrieve dag info!");
    web::Json(api_res)
}

#[get("/hardInboxFiles")]
pub async fn get_inbox_files() -> web::Json<VString>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let inbox: String = application().inbox_path();
        let files: VString = list_exact_files(&inbox, "");
        let mut out: VString = vec![];
        for a_file in files
        {
            let file_dtl = a_file.split("/").collect::<Vec<&str>>();
            let name = file_dtl[file_dtl.len() - 1].to_string();
            out.push(name);
        }
        out
    }).await.expect("Failed in retrieve inbox info!");
    web::Json(api_res)
}

#[get("/listParsingQ")]
pub async fn get_parsing_q() -> web::Json<QVDRecordsT>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let records = search_parsing_q(
            vec![],
            vec!["pq_id", "pq_type", "pq_code", "pq_sender", "pq_connection_type",
                 "pq_receive_date", "pq_prerequisites", "pq_parse_attempts", "pq_v_status",
                 "pq_creation_date", "pq_insert_date", "pq_last_modified"],
            vec![],
            0);
        records
    }).await.expect("Failed in retrieve parsing Q!");
    web::Json(api_res)
}

#[get("/listMissedBlocks")]
pub async fn get_missed_blocks() -> web::Json<QVDRecordsT>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let records = list_missed_blocks(
            vec![],
            vec![],
            vec![],
            0);
        records
    }).await.expect("Failed in retrieve missed blocks!");
    web::Json(api_res)
}

#[get("/listSendingQ")]
pub async fn get_sending_q() -> web::Json<QVDRecordsT>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let records = fetch_from_sending_q(
            vec![],
            vec![],
            vec![],
            0);
        records
    }).await.expect("Failed in retrieve missed blocks!");
    web::Json(api_res)
}

#[get("/hardOutboxFiles")]
pub async fn get_outbox_files() -> web::Json<VString>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let outbox: String = application().outbox_path();
        let files: VString = list_exact_files(&outbox, "");
        let mut out: VString = vec![];
        for a_file in files
        {
            let file_dtl = a_file.split("/").collect::<Vec<&str>>();
            let name = file_dtl[file_dtl.len() - 1].to_string();
            out.push(name);
        }
        out
    }).await.expect("Failed in retrieve outbox info!");
    web::Json(api_res)
}

#[get("/listLeavesByKV")]
pub async fn get_leaves_by_kv() -> web::Json<HashMap<String, LeaveBlock>>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let leaves = get_leave_blocks(&"".to_string());
        leaves
    }).await.expect("Failed in retrieve leaves list by KV!");
    web::Json(api_res)
}

#[get("/listFreshLeaves")]
pub async fn list_fresh_leaves() -> web::Json<HashMap<String, LeaveBlock>>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let fresh_leaves = get_fresh_leaves();
        println!(" fresh leaves {:?}", fresh_leaves);
        fresh_leaves
    }).await.expect("Failed in retrieve fresh leaves!");
    web::Json(api_res)
}