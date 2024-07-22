// use postgres::{Client, NoTls};
// use dotenv::dotenv;
// use std::env;

#[allow(warnings, unused)]
mod prisma;

use prisma::PrismaClient;
// use prisma_client_rust::NewClientError;

#[tokio::main]
async fn main(){
    // dotenv().ok();
    // let database_url = env::var("DATABASE_URL")
    //     .expect("DATABASE_URL must be set");
    // let _connection = Client::connect(&database_url, NoTls);
    // println!("Connection to the database established!");

    let _client = PrismaClient::_builder().build().await;
}