use actix_web::{get, post, web, Responder};
use crate::lib::custom_types::JSonObject;
use crate::lib::machine::machine_profile::{get_current_profile, get_profile_from_db, get_profiles_list, MachineProfile};
use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::cutils::{controlled_str_to_json, remove_quotes};
use crate::{constants, dlog, machine};

#[get("/profile")]
pub async fn profile() -> web::Json<MachineProfile> {
    let res = tokio::task::spawn_blocking(|| {
        get_current_profile()
    }).await.expect("Task panicked");
    web::Json(res)
}

#[get("/profiles")]
pub async fn profiles() -> web::Json<Vec<JSonObject>> {
    let res = tokio::task::spawn_blocking(|| {
        get_profiles_list()
    }).await.expect("Task panicked");
    web::Json(res)
}

// #[post("/")]
// async fn example() -> HttpResponse {
//     HttpResponse::Ok().finish()
// }

/// Struct to receive user input.
#[derive(Serialize, Deserialize)]
pub struct NewPost {
    pub content: String,
}

#[post("/saveSettings")]
pub async fn save_machine_settings(post: String) -> impl Responder
{
    let api_res = tokio::task::spawn_blocking(move || {
        let (_status, request) = controlled_str_to_json(&post);
        println!("New POST request to create a post! request {:?}", request);

        let mp_code = remove_quotes(&request["mpCode"]);
        let message = format!("updating machine profile {}", mp_code);
        println!("{}", message);
        dlog(
            &message,
            constants::Modules::App,
            constants::SecLevel::Info);

        let (_status, mut the_profile) = get_profile_from_db(&mp_code);

        the_profile.m_mp_settings.m_public_email.m_address
            = remove_quotes(&request["publicEmailAddress"]);
        the_profile.m_mp_settings.m_public_email.m_password
            = remove_quotes(&request["publicEmailPass"]);
        the_profile.m_mp_settings.m_public_email.m_fetching_interval_by_minute
            = remove_quotes(&request["publicEmailFetchInterval"]);
        the_profile.m_mp_settings.m_public_email.m_income_imap
            = remove_quotes(&request["publicEmailIMAP"]);
        the_profile.m_mp_settings.m_public_email.m_income_pop3
            = remove_quotes(&request["publicEmailPOP3"]);
        the_profile.m_mp_settings.m_public_email.m_incoming_mail_server
            = remove_quotes(&request["publicEmailServer"]);

        if request["ToS"].as_bool().unwrap()
        {
            the_profile.m_mp_settings.m_term_of_services = constants::YES.to_string();
        }else{
            the_profile.m_mp_settings.m_term_of_services = constants::NO.to_string();
        }

        machine().m_profile = the_profile;
        machine().save_settings();

        let api_res = json!({
            "status": true,
            "message": "Profile updated".to_string(),
            "info": json!({}),
        });
        api_res
    }).await.expect("saveSettings panicked");
    web::Json(api_res)
}