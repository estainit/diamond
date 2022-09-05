use actix_web::{get, web};
use crate::{machine};
use crate::lib::custom_types::{QVDRecordsT};
use crate::lib::database::tables::C_MACHINE_WALLET_ADDRESSES_FIELDS;
use crate::lib::wallet::wallet_address_handler::get_addresses_list;
use crate::lib::wallet::wallet_coins::get_coins_list;


#[get("/getAddresses")]
pub async fn get_addresses() -> web::Json<QVDRecordsT>
{
    let api_res = tokio::task::spawn_blocking(|| {
        let (wallet_controlled_accounts, details) = get_addresses_list(
            &machine().get_selected_m_profile(),
            Vec::from(C_MACHINE_WALLET_ADDRESSES_FIELDS),
            true);
        wallet_controlled_accounts
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

