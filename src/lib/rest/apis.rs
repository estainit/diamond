use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use crate::lib::constants;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body(constants::SOCIETY_NAME)
    // HttpResponse::Ok().body("hello world")
}
pub async fn run_web_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}