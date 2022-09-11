use actix_web::{get, web, App, HttpServer};
use actix_cors::Cors;
use crate::{application, dlog};
use crate::lib::constants;
use crate::lib::custom_types::JSonObject;
use crate::lib::machine::machine_neighbor::{add_a_new_neighbor_by_email, handshake_neighbor};
use crate::lib::machine::machine_profile::MachineProfile;
use crate::lib::rest::profile_apis::{save_machine_settings, profile, profiles};
use serde::{Serialize, Deserialize};
use crate::lib::rest::monitor_apis::{dag_info, get_inbox_files, get_leaves_by_kv, get_missed_blocks, get_outbox_files, get_parsing_q, get_sending_q, is_synchronizing, list_fresh_leaves};
use crate::lib::rest::wallet_apis::{create_basic_1of1_address, create_basic_2of3_address, create_basic_3of5_address, get_addresses, get_coins, refresh_w_coins, sign_trx_and_push_to_buffer};

pub async fn run_web_server() -> std::io::Result<()> {
    let webserver = application().web_server_address().clone();
    let webserver = webserver.split(":").collect::<Vec<&str>>();
    let host_ = format!("{}", webserver[0]);
    let port_ = format!("{}", webserver[1]).parse::<u16>().unwrap();
    HttpServer::new(|| {
        // FIXME: The permissive constructor should not be used in production.
        let cors = Cors::permissive();
        // default()
        //     .allowed_methods(vec!["GET", "POST"])
        //     .allowed_origin("*")
        //     .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        //     .allowed_header(http::header::CONTENT_TYPE)
        //     .allowed_header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN)
        //     .max_age(3600);

        App::new()
            .wrap(cors)
            .service(hi)
            .service(hello)

            // profile APIs
            .service(profile)
            .service(profiles)
            .service(save_machine_settings)

            // Monitor APIs
            .service(dag_info)
            .service(get_inbox_files)
            .service(get_parsing_q)
            .service(get_missed_blocks)
            .service(get_sending_q)
            .service(get_outbox_files)
            .service(get_leaves_by_kv)
            .service(list_fresh_leaves)
            .service(is_synchronizing)

            // Wallet APIs
            .service(get_addresses)
            .service(refresh_w_coins)
            .service(get_coins)
            .service(create_basic_1of1_address)
            .service(create_basic_2of3_address)
            .service(create_basic_3of5_address)
            .service(sign_trx_and_push_to_buffer)
    })
        .bind((host_, port_))?
        .run()
        .await
}

#[derive(Serialize, Deserialize)]
pub struct APIResponse {
    pub status: bool,
    pub message: String,
    pub info: JSonObject,
}

#[get("/hello")]
pub async fn hello() -> web::Json<MachineProfile> {
    let t = tokio::task::spawn_blocking(|| {
        crate::lib::machine::machine_profile::get_profile_from_db(&constants::DEFAULT)
    }).await.expect("Task panicked");
    web::Json(t.1)
}

#[get("/")]
pub async fn hi() -> web::Json<Vec<String>> {
    let t = tokio::task::spawn_blocking(|| {
        vec!["HI iiiii".to_string()]
    }).await.expect("Task panicked");
    web::Json(t)
}


// frontend methods
// use get_neighbors to get list of neighbors


pub fn do_handshake_by_email(neighbor_email: String) -> (bool, String)
{
    // the node only has an email address of new neighbor
    // so inserts it as a new neighbor
    let (status, msg, neighbor_id) = add_a_new_neighbor_by_email(neighbor_email);
    if !status
    { return (false, msg); }

    // then sends it a handshake request
    return do_handshake(neighbor_id);
}

pub fn do_handshake(neighbor_id: i64) -> (bool, String)
{
    let (status, message) = handshake_neighbor(neighbor_id, constants::PUBLIC);
    if status
    {
        dlog(
            &format!("Handshake Done neighbor({})", neighbor_id),
            constants::Modules::App,
            constants::SecLevel::Info);
    } else {
        dlog(
            &format!("Failed Handshake neighbor({})", neighbor_id),
            constants::Modules::App,
            constants::SecLevel::Error);
    }
    return (status, message);
}