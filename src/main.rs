mod auth;
mod prisma;

use std::sync::Arc; 

use prisma::*;

use log::{info, warn, error, debug};
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");
    debug!("This is a debug message");


    let prisma_client = PrismaClient::_builder().build().await.unwrap();
    let prisma_client = Arc::new(prisma_client);
    HttpServer::new(move || {
        App::new()
            .service(hello)
            .app_data(web::Data::new(Arc::clone(&prisma_client)))
            .service(
                web::scope("/api/auth")
                .configure(auth::routes::init_routes)
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}