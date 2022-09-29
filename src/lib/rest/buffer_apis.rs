use actix_web::{get, post, Responder, web};
use postgres::types::ToSql;
use serde_json::json;
use crate::cutils::{controlled_str_to_json, remove_quotes};
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, simple_eq_clause};
use crate::machine;


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
        if status
        {
            let res = json!({
                "status": true,
                "message": format!("Requested document was deleted, {} ", bd_id),
                "info": json!({}),
            });
            res
        } else {
            let res = json!({
                "status": false,
                "message": format!("Failed in delete {}", bd_id),
                "info": json!({}),
            });
            res
        }
    }).await.expect("sign transaction panicked");
    web::Json(api_res)
}

#[get("/broadcastBlock")]
pub async fn broadcast_the_block() -> impl Responder
{
    let api_res = tokio::task::spawn_blocking(|| {
        let (status, msg) = machine().broadcast_block(&"".to_string(), &"".to_string());
        if !status
        {
            let res = json!({
                "status": false,
                "message": msg,
                "info": json!({}),
            });
            res
        } else {
            let res = json!({
                "status": true,
                "message": format!("Block was broadcast. {}", 88),
                "info": json!({}),
            });
            res
        }
    }).await.expect("Failed in create Basic 1/1 address!");
    web::Json(api_res)
}

