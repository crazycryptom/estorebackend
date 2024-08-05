use super::handler::*;
use actix_web::web;

pub fn client_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/orders").route(web::get().to(get_orders)));
    cfg.service(web::resource("/orders").route(web::post().to(place_order)));
}
