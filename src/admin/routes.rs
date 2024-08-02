use actix_web::web;
use super::handler::*;

pub fn admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users")
        .route(web::get().to(get_users))
    );
    cfg.service(
        web::resource("/users")
        .route(web::put().to(update_user_role))
    );
}