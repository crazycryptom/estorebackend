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
    pub role: String,
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub is_admin: bool,
}

#[derive(Deserialize)]
pub struct Passwords {
    pub oldpassword: String,
    pub newpassword: String,
}

#[derive(Deserialize)]

pub struct UpdateProfile {
    pub username: String,
    pub email: String,
    pub firstname: String,
    pub lastname: String,
}

#[derive(Deserialize)]
pub struct GetRecoveryKeyPayload {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordPayload {
    pub recoverykey: String,
    pub email: String,
    pub newpassword: String,
}