use actix_web::{get, web, App, HttpServer};
use crate::dlog;
use crate::lib::constants;
use crate::lib::machine::machine_neighbor::{add_a_new_neighbor_by_email, handshake_neighbor};

// #[get("/")]
// pub async fn hello() -> impl Responder {
//     HttpResponse::Ok().body(constants::SOCIETY_NAME)
//     // HttpResponse::Ok().body("hello world")
// }

pub async fn run_web_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

use crate::lib::machine::machine_profile::MachineProfile;

#[get("/")]
pub async fn hello() -> web::Json<MachineProfile> {
    let t = tokio::task::spawn_blocking(|| {
        crate::lib::machine::machine_profile::MachineProfile::get_profile_from_db(&constants::DEFAULT)
    }).await.expect("Task panicked");
    web::Json(t.1)
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