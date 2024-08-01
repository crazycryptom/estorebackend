use actix_web::web;
use super::handler::*;
use crate::utils::Authentication;

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
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
    cfg.service(
        web::resource("/update-profile")
        .wrap(Authentication)
        .route(web::put().to(update_profile))
    );
    cfg.service(
        web::resource("/recovery-key")
        .wrap(Authentication)
        .route(web::post().to(recovery_key))
    );
    cfg.service(
        web::resource("/reset-password")
        .route(web::post().to(reset_password))
    );
}