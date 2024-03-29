use std::collections::HashMap;
use actix_web::{get, post, Responder, web};
use serde_json::json;
use crate::{constants, dlog, machine};
use crate::constants::{MONEY_MAX_DIVISION};
use crate::cutils::{controlled_str_to_json, remove_quotes};
use crate::lib::custom_types::{CMPAIValueT, QV2DicT, QVDRecordsT, VString};
use crate::lib::wallet::get_addresses_list::get_addresses_list;
use crate::lib::wallet::wallet_address_handler::{create_and_insert_new_address_in_wallet};
use crate::lib::wallet::wallet_coins::get_coins_list;
use crate::lib::wallet::wallet_signer::wallet_signer;


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
        let coins = get_coins_list(false);
        let mut new_coins: QVDRecordsT = vec![];
        for mut a_coin in coins
        {
            a_coin.insert(
                "coin_code".to_string(),
                format!("{}:{}", a_coin["wf_trx_hash"], a_coin["wf_o_index"]));
            new_coins.push(a_coin);
        }
        new_coins
    }).await.expect("Failed in retrieve fresh leaves!");
    web::Json(api_res)
}

#[get("/refreshCoins")]
pub async fn refresh_w_coins() -> web::Json<QVDRecordsT>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let coins = get_coins_list(true);
        let mut new_coins: QVDRecordsT = vec![];
        for mut a_coin in coins
        {
            a_coin.insert(
                "coin_code".to_string(),
                format!("{}:{}", a_coin["wf_trx_hash"], a_coin["wf_o_index"]));
            new_coins.push(a_coin);
        }
        println!("coinssss: {:?}", new_coins);
        new_coins
    }).await.expect("Failed in retrieve fresh leaves!");
    web::Json(api_res)
}

#[get("/createBasic1of1Address")]
pub async fn create_basic_1of1_address() -> web::Json<(bool, String)>
{
    let api_res = tokio::task::spawn_blocking(|| {
        return create_and_insert_new_address_in_wallet(
            constants::signature_types::BASIC,
            "1/1",
            constants::CURRENT_SIGNATURE_VERSION);

        // let (status, unlock_doc) = create_a_new_address(
        //     constants::signature_types::BASIC,
        //     "1/1",
        //     "0.0.1");
        // if !status
        // {
        //     err_msg = format!("Couldn't create an ECDSA 1of1 key pairs");
        //     return (false, err_msg);
        // }
        // let mp_code = machine().get_selected_m_profile().clone();
        // let now_ = application().now();
        // let w_address = WalletAddress {
        //     m_mp_code: mp_code,
        //     m_address: unlock_doc.m_account_address.clone(),
        //     m_title: "Basic address (1/1 signature) ver(0.0.1)".to_string(),
        //     m_unlock_doc: unlock_doc,
        //     m_creation_date: now_,
        // };
        // let (status, msg) = insert_address(&w_address);
        // return (status, msg);
    }).await.expect("Failed in create Basic 1/1 address!");
    web::Json(api_res)
}

#[get("/createBasic2of3Address")]
pub async fn create_basic_2of3_address() -> web::Json<(bool, String)>
{
    let api_res = tokio::task::spawn_blocking(|| {
        return create_and_insert_new_address_in_wallet(
            constants::signature_types::BASIC,
            "2/3",
            constants::CURRENT_SIGNATURE_VERSION);

        // let err_msg: String;
        // let (status, unlock_doc) = create_a_new_address(
        //     constants::signature_types::BASIC,
        //     "2/3",
        //     constants::CURRENT_SIGNATURE_VERSION);
        // if !status
        // {
        //     err_msg = format!("Couldn't create an ECDSA 1of1 key pairs");
        //     return (false, err_msg);
        // }
        // let mp_code = machine().get_selected_m_profile().clone();
        // let now_ = application().now();
        // let w_address = WalletAddress {
        //     m_mp_code: mp_code,
        //     m_address: unlock_doc.m_account_address.clone(),
        //     m_title: "Basic address (2/3 signature) ver(0.0.1)".to_string(),
        //     m_unlock_doc: unlock_doc,
        //     m_creation_date: now_,
        // };
        // let (status, msg) = insert_address(&w_address);
        // return (status, msg);
    }).await.expect("Failed in create Basic 1/1 address!");
    web::Json(api_res)
}

#[get("/createBasic3of5Address")]
pub async fn create_basic_3of5_address() -> web::Json<(bool, String)>
{
    let api_res = tokio::task::spawn_blocking(|| {
        return create_and_insert_new_address_in_wallet(
            constants::signature_types::BASIC,
            "3/5",
            constants::CURRENT_SIGNATURE_VERSION);

        // let err_msg: String;
        // let (status, unlock_doc) = create_a_new_address(
        //     constants::signature_types::BASIC,
        //     "3/5",
        //     constants::CURRENT_SIGNATURE_VERSION);
        // if !status
        // {
        //     err_msg = format!("Couldn't create an ECDSA 1of1 key pairs");
        //     return (false, err_msg);
        // }
        // let mp_code = machine().get_selected_m_profile().clone();
        // let now_ = application().now();
        // let w_address = WalletAddress {
        //     m_mp_code: mp_code,
        //     m_address: unlock_doc.m_account_address.clone(),
        //     m_title: "Basic address (3/5 signature) ver(0.0.1)".to_string(),
        //     m_unlock_doc: unlock_doc,
        //     m_creation_date: now_,
        // };
        // let (status, msg) = insert_address(&w_address);
        // return (status, msg);
    }).await.expect("Failed in create Basic 1/1 address!");
    web::Json(api_res)
}


#[post("/signTrxAndPushToBuffer")]
pub async fn sign_trx_and_push_to_buffer(post: String) -> impl Responder
{
    let api_res = tokio::task::spawn_blocking(move || {
        let (_status, request) = controlled_str_to_json(&post);
        println!("New POST request to create a post! request {:?}", request);

        let d_comment = remove_quotes(&request["dComment"]);
        if request["txAmount"].is_null() || request["txAmount"] == "\"\""
        {
            return json!({
                "status": false,
                "message": "Missed transaction amount!".to_string(),
                "info": json!({}),
            });
        }
        let trx_amount = remove_quotes(&request["txAmount"]).parse::<CMPAIValueT>().unwrap();

        if request["txRecipient"].is_null() || request["txRecipient"] == "\"\""
        {
            return json!({
                "status": false,
                "message": "Missed recipient!".to_string(),
                "info": json!({}),
            });
        }
        let trx_recipient = remove_quotes(&request["txRecipient"]);

        if request["txFeeCalcMethod"].is_null() || request["txFeeCalcMethod"] == "\"\""
        {
            return json!({
                "status": false,
                "message": "Missed Fee Calculate Method!".to_string(),
                "info": json!({}),
            });
        }
        let trx_fee_calc_method = remove_quotes(&request["txFeeCalcMethod"]);

        let mut trx_fee: CMPAIValueT = 0;
        if trx_fee_calc_method == constants::transaction_fee_calculate_methods::EXACT_FEE.to_string()
        {
            if request["txFee"].is_null() || request["txFee"] == "\"\"" || request["txFee"] == "0"
            {
                return json!({
                    "status": false,
                    "message": "Missed transaction fee!".to_string(),
                    "info": json!({}),
                });
            }
            trx_fee = (&request["txFee"].as_f64().unwrap() * constants::MONEY_MAX_DIVISION as f64) as CMPAIValueT;
        }

        let trx_change_back_mod = remove_quotes(&request["changeBackMod"]);
        let trx_change_back_address = remove_quotes(&request["changeBackAddress"]);
        let selected_coins = request["selectedCoins"].as_array().unwrap();

        println!("selected_coins,1,,,,,{:?}", selected_coins);
        let selected_coins = selected_coins
            .iter()
            .map(|x| remove_quotes(&x))
            .collect::<VString>();
        println!("selected_coins,2,,,,,{:?}", selected_coins);


        if selected_coins.len() == 0
        {
            return json!({
                "status": false,
                "message": "No coin selected to be spent".to_string(),
                "info": json!({}),
            });
        }
        // let selected_coins = convert_comma_separated_string_to_string_vector(&selected_coins);

        let msg: String;
        let trx_outputs_bill_amount = "0"; // the amount of output(s). zero means one output for all amount, while 1000000 means each output must be a million mPAI
        // TODO: add some control on input fields

        let bill_size: CMPAIValueT;
        if trx_outputs_bill_amount == "0"
        {
            bill_size = 0;
        } else {
            bill_size = trx_outputs_bill_amount.to_string().parse::<CMPAIValueT>().unwrap();
        }

        let (sign_res, sign_status_msg) = wallet_signer(
            &selected_coins,
            trx_amount * MONEY_MAX_DIVISION,
            &trx_fee_calc_method,
            trx_fee,
            &trx_recipient,
            &trx_change_back_mod,
            &trx_change_back_address,
            bill_size,
            d_comment,
        );
        if !sign_res
        {
            msg = format!(
                "Failed in transaction sign, {} ",
                sign_status_msg);
            dlog(
                &msg,
                constants::Modules::Trx,
                constants::SecLevel::Error);
            return json!({
                "status": false,
                "message": "Failed in transaction sign or send to machine buffer".to_string(),
                "info": json!({}),
            });
        }

        let api_res = json!({
            "status": true,
            "message": format!("Transaction signed, {} ", sign_status_msg),
            "info": json!({}),
        });
        api_res
    }).await.expect("sign transaction panicked");
    web::Json(api_res)
}