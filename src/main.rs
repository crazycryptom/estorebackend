mod admin;
mod auth;
mod prisma;
mod utils;
mod general;

use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use log::{debug, error, info, warn};
use prisma::*;
use std::sync::Arc;
use utils::Authentication;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let prisma_client = PrismaClient::_builder().build().await.unwrap();
    let prisma_client = Arc::new(prisma_client);
    HttpServer::new(move || {
        let cors = Cors::default().allowed_methods(vec!["GET", "POST", "PUT", "DELETE"]);
        App::new()
            .wrap(cors)
            .service(hello)
            .app_data(web::Data::new(Arc::clone(&prisma_client)))
            .service(web::scope("/api/auth").configure(auth::routes::auth_routes))
            .service(
                web::scope("api/admin")
                    .wrap(Authentication)
                    // .wrap(Authorization)
                    .configure(admin::routes::admin_routes),
            )
            .service(
                web::scope("api")
                .configure(general::routes::general_routes)
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
