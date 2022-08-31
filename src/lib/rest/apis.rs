use actix_web::{get, web, App, HttpServer, http};
use actix_cors::Cors;
use warp::Filter;
use crate::dlog;
use crate::lib::constants;
use crate::lib::custom_types::JSonObject;
use crate::lib::machine::machine_neighbor::{add_a_new_neighbor_by_email, handshake_neighbor};
use crate::lib::machine::machine_profile::MachineProfile;
use crate::lib::rest::profile_apis::{profile, profiles};

pub async fn run_web_server() -> std::io::Result<()> {
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
            .service(profile)
            .service(profiles)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
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