use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}