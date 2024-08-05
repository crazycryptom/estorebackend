use actix_web::web;
use super::handler::*;

pub fn general_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/products")
        .route(web::get().to(get_products))
    );
    cfg.service(
        web::resource("/categories")
        .route(web::get().to(get_categories))
    );
}