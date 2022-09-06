use std::collections::HashMap;
use actix_web::{get, web};
use crate::{application, constants, machine};
use crate::lib::address::address_handler::create_a_new_address;
use crate::lib::custom_types::{QV2DicT, QVDRecordsT};
use crate::lib::wallet::get_addresses_list::get_addresses_list;
use crate::lib::wallet::wallet_address_handler::{insert_address, WalletAddress};
use crate::lib::wallet::wallet_coins::get_coins_list;


#[get("/getAddresses")]
pub async fn get_addresses() -> web::Json<QV2DicT>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let (wallet_controlled_accounts, mut details) = get_addresses_list(
            &machine().get_selected_m_profile(),
            vec!["wa_id", "wa_mp_code", "wa_address", "wa_title", "wa_creation_date"],
            true);
        for an_address in &wallet_controlled_accounts
        {
            if details.contains_key(&an_address["wa_address"])
            {
                let mut tmp_elm = details[&an_address["wa_address"]].clone();
                tmp_elm.insert("wa_id".to_string(), an_address["wa_id"].clone());
                tmp_elm.insert("wa_title".to_string(), an_address["wa_title"].clone());
                tmp_elm.insert("wa_creation_date".to_string(), an_address["wa_creation_date"].clone());
                details.insert(an_address["wa_address"].clone(), tmp_elm);
            } else {
                let tmp_elm = HashMap::from([
                    ("mat_sum".to_string(), "0".to_string()),
                    ("mat_count".to_string(), "0".to_string()),
                    ("unmat_sum".to_string(), "0".to_string()),
                    ("unmat_count".to_string(), "0".to_string()),
                    ("wa_id".to_string(), an_address["wa_id"].clone()),
                    ("wa_title".to_string(), an_address["wa_title"].clone()),
                    ("wa_creation_date".to_string(), an_address["wa_creation_date"].clone()),
                ]);
                details.insert(an_address["wa_address"].clone(), tmp_elm);
            }
        }
        details
    }).await.expect("Failed in retrieve fresh leaves!");
    web::Json(api_res)
}

#[get("/getCoins")]
pub async fn get_coins() -> web::Json<QVDRecordsT>
{
    let api_res = tokio::task::spawn_blocking(|| {
        get_coins_list(false)
    }).await.expect("Failed in retrieve fresh leaves!");
    web::Json(api_res)
}

#[get("/refreshCoins")]
pub async fn refresh_w_coins() -> web::Json<QVDRecordsT>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let coins = get_coins_list(true);
        println!("coinssss: {:?}", coins);
        coins
    }).await.expect("Failed in retrieve fresh leaves!");
    web::Json(api_res)
}

#[get("/createBasic1of1Address")]
pub async fn create_basic_1of1_address() -> web::Json<(bool, String)>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let err_msg: String;
        let (status, unlock_doc) = create_a_new_address(
            constants::signature_types::BASIC,
            "1/1",
            "0.0.1");
        if !status
        {
            err_msg = format!("Couldn't create an ECDSA 1of1 key pairs");
            return (false, err_msg);
        }
        let mp_code = machine().get_selected_m_profile().clone();
        let now_ = application().now();
        let w_address = WalletAddress {
            m_mp_code: mp_code,
            m_address: unlock_doc.m_account_address.clone(),
            m_title: "Basic address (1/1 signature) ver(0.0.1)".to_string(),
            m_unlock_doc: unlock_doc,
            m_creation_date: now_,
        };
        let (status, msg) = insert_address(&w_address);
        return (status, msg);
    }).await.expect("Failed in create Basic 1/1 address!");
    web::Json(api_res)
}

#[get("/createBasic2of3Address")]
pub async fn create_basic_2of3_address() -> web::Json<(bool, String)>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let err_msg: String;
        let (status, unlock_doc) = create_a_new_address(
            constants::signature_types::BASIC,
            "2/3",
            "0.0.1");
        if !status
        {
            err_msg = format!("Couldn't create an ECDSA 1of1 key pairs");
            return (false, err_msg);
        }
        let mp_code = machine().get_selected_m_profile().clone();
        let now_ = application().now();
        let w_address = WalletAddress {
            m_mp_code: mp_code,
            m_address: unlock_doc.m_account_address.clone(),
            m_title: "Basic address (2/3 signature) ver(0.0.1)".to_string(),
            m_unlock_doc: unlock_doc,
            m_creation_date: now_,
        };
        let (status, msg) = insert_address(&w_address);
        return (status, msg);
    }).await.expect("Failed in create Basic 1/1 address!");
    web::Json(api_res)
}

#[get("/createBasic3of5Address")]
pub async fn create_basic_3of5_address() -> web::Json<(bool, String)>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let err_msg: String;
        let (status, unlock_doc) = create_a_new_address(
            constants::signature_types::BASIC,
            "3/5",
            "0.0.1");
        if !status
        {
            err_msg = format!("Couldn't create an ECDSA 1of1 key pairs");
            return (false, err_msg);
        }
        let mp_code = machine().get_selected_m_profile().clone();
        let now_ = application().now();
        let w_address = WalletAddress {
            m_mp_code: mp_code,
            m_address: unlock_doc.m_account_address.clone(),
            m_title: "Basic address (3/5 signature) ver(0.0.1)".to_string(),
            m_unlock_doc: unlock_doc,
            m_creation_date: now_,
        };
        let (status, msg) = insert_address(&w_address);
        return (status, msg);
    }).await.expect("Failed in create Basic 1/1 address!");
    web::Json(api_res)
}

