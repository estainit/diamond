use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use crate::dlog;
use crate::lib::constants;
use crate::lib::machine::machine_neighbor::handshake_neighbor;

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
    let t=tokio::task::spawn_blocking(|| {
        crate::lib::machine::machine_profile::MachineProfile::get_profile_from_db(&constants::DEFAULT)
    }).await.expect("Task panicked");
    web::Json(t.1)
}





// frontend methods
// use get_neighbors to get list of neighbors


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