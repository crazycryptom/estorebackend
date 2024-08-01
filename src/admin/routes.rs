use actix_web::web;
use super::handler::*;
use crate::utils::Authentication;

pub fn admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users")
        .wrap(Authentication)
        .route(web::get().to(get_users))
    );
}