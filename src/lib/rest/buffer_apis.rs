use std::collections::HashMap;
use actix_web::{get, post, Responder, web};
use postgres::types::ToSql;
use serde_json::json;
use crate::{constants, dlog, machine};
use crate::constants::{MONEY_MAX_DIVISION};
use crate::cutils::{controlled_str_to_json, remove_quotes};
use crate::lib::custom_types::{CMPAIValueT, QV2DicT, QVDRecordsT, VString};
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, simple_eq_clause};
use crate::lib::wallet::get_addresses_list::get_addresses_list;
use crate::lib::wallet::wallet_address_handler::{create_and_insert_new_address_in_wallet};
use crate::lib::wallet::wallet_coins::get_coins_list;


#[get("/getBuffer")]
pub async fn get_buffered_docs() -> impl Responder
{
    let api_res = tokio::task::spawn_blocking(|| {
        let mp_code = machine().get_selected_m_profile();
        let buffered_docs = machine().search_buffered_docs(
            vec![
                simple_eq_clause("bd_mp_code", &mp_code),
            ],
            vec!["bd_id", "bd_mp_code", "bd_insert_date", " bd_doc_hash", "bd_doc_type",
                 "bd_doc_class", "bd_dp_cost", "bd_doc_len"],
            vec![
                &OrderModifier { m_field: "bd_dp_cost", m_order: "DESC" },
                &OrderModifier { m_field: "bd_insert_date", m_order: "ASC" },
            ],
            0,
        );

        let res = json!({
                "status": true,
                "message": format!("Found {} Buffered document!", buffered_docs.len()),
                "info": buffered_docs,
            });
        res
    }).await.expect("Failed in create Basic 1/1 address!");
    web::Json(api_res)
}


#[post("/delBuffDoc")]
pub async fn delete_buffered_doc(post: String) -> impl Responder
{
    let api_res = tokio::task::spawn_blocking(move || {
        let (_status, request) = controlled_str_to_json(&post);
        println!("New delete-buffered-doc request {:?}", request);

        let bd_id = remove_quotes(&request["bdId"]).parse::<i64>().unwrap();

        let status = machine().remove_from_buffer(vec![ModelClause {
            m_field_name: "bd_id",
            m_field_single_str_value: &bd_id as &(dyn ToSql + Sync),
            m_clause_operand: "=",
            m_field_multi_values: vec![],
        }]);

        let res = json!({
            "status": true,
            "message": format!("Requested document was deleted, {} ", 22),
            "info": json!({}),
        });
        res
    }).await.expect("sign transaction panicked");
    web::Json(api_res)
}

