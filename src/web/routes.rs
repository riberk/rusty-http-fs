mod info;

use actix_web::web;

use super::app_data::AppData;

pub fn configure<D: AppData + 'static>(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/info/v1", web::get().to(info::info::<D>));
}
