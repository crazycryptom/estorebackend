use actix_web::web;
use crate::auth::handler::{register_user, login_user, change_pass};
use super::utils::Authentication;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/register")
            .route(web::post().to(register_user))
    );
    cfg.service(
        web::resource("/login")
            .route(web::post().to(login_user))
    );
    cfg.service(
        web::resource("/password-change")
        .wrap(Authentication)
        .route(web::put().to(change_pass))
    );
}