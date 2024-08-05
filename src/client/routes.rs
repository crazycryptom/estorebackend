use super::handler::*;
use crate::Authentication;
use actix_web::web;

pub fn client_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/orders")
            .wrap(Authentication)
            .route(web::post().to(place_order)),
    );
}
