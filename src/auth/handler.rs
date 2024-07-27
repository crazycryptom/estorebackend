use actix_web::{web, Responder, HttpResponse};
use serde_json::json;
use crate::prisma::PrismaClient;
use crate::prisma::*;
use crate::auth::model::{RegisterUser, UserResponse, LoginUser, JWTResponse, Claims};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use std::sync::Arc;
use dotenv::dotenv;
use std::env;
use chrono::Utc;

fn get_secret_key() -> Vec<u8> {
    dotenv().ok();
    env::var("JWT_SECRET")
        .expect("SECRET_KEY must be set")
        .into_bytes()
}

pub async fn register_user(
    user: web::Json<RegisterUser>,
    prisma_client: web::Data<Arc<PrismaClient>>
) -> impl Responder {
    // Check if the user already exists
    match prisma_client.user().find_unique(
        user::email::equals(user.email.clone())
    ).exec().await {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().json(json!({"error": "User already exists"}));
        }
        Ok(None) => {
            let hashed_password = match hash(user.password.clone(), DEFAULT_COST) {
                Ok(p) => p,
                Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Failed to hash password"})),
            };
            // Proceed to create a new user
            let role_type = match user.role.as_str() {
                "admin" => RoleType::Admin,
                _ => RoleType::Client,
            };

            match prisma_client.user().create(
                user.username.clone(),
                user.first_name.clone(),
                user.last_name.clone(),
                user.email.clone(),
                hashed_password.clone(),
                role_type.clone(),
                vec![],
            ).exec().await {
                Ok(new_user) => {
                    // Return the created user
                    return HttpResponse::Created().json(UserResponse {
                        id: new_user.id,
                        username: new_user.display_name,
                        email: new_user.email,
                        first_name: new_user.first_name,
                        last_name: new_user.last_name,
                    });
                }
                Err(err) => {
                    return HttpResponse::InternalServerError().json(json!({"error": format!("Failed to create user: {}", err)}));
                }
            };
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(json!({"error": format!("Failed to query user: {}", err)}));
        }
    }
}

pub async fn login_user(
    user: web::Json<LoginUser>,
    prisma_client: web::Data<Arc<PrismaClient>>
) -> impl Responder {
    match prisma_client.user().find_unique(
        user::email::equals(user.email.clone())
    ).exec().await {
        Ok(Some(user_record)) => {
            if let Ok(valid) = verify(&user.password, &user_record.password) {
                if valid {
                    let exp = chrono::Utc::now()
                        .checked_add_signed(chrono::Duration::days(1))
                        .expect("valid timestamp")
                        .timestamp() as usize;

                    let claims = Claims {
                        sub: user_record.id.clone(),
                        exp,
                    };

                    let secret = get_secret_key();
                    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(&secret)) {
                        Ok(t) => t,
                        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Failed to generate token"})),
                    };

                    return HttpResponse::Ok().json(json!({
                        "token": token,
                        "user": UserResponse {
                            id: user_record.id,
                            username: user_record.display_name,
                            email: user_record.email,
                            first_name: user_record.first_name,
                            last_name: user_record.last_name,
                        }
                    }));
                }
            }
            HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"}))
        }
        Ok(None) => HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"})),
        Err(err) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to query user: {}", err)})),
    }
}