use postgres::{Client, NoTls};
use dotenv::dotenv;
use std::env;

fn main(){
    dotenv().ok();
    openssl_probe::init_ssl_cert_env_vars();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let _connection = Client::connect(&database_url, NoTls);
    println!("Connection to the database established!");
}