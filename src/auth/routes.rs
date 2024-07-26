use actix_web::web;
use crate::auth::handler::register_user;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/api/auth/register")
            .route(web::post().to(register_user))
    );
}