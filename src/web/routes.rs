pub mod status;

use actix_web::web;

use crate::utils::time::Time;

pub fn configure<TTime: Time + 'static>(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/api/info/v1",
        web::get().to(crate::endpoints::info::<TNow>),
    )
    .route(
        "/auth/token",
        web::post().to(crate::endpoints::token::<TNow>),
    )
    .route("/health", web::route().to(health));
}