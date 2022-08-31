use actix_web::{get, web, App, HttpServer, http};
use crate::lib::custom_types::JSonObject;
use crate::lib::machine::machine_profile::{get_current_profile, get_profiles_list, MachineProfile};

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
