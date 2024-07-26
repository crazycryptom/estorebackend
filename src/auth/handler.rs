use actix_web::{web, Responder, HttpResponse};
use serde_json::json;
use crate::prisma::PrismaClient;
use crate::prisma::*;
use crate::auth::model::{RegisterUser, UserResponse};
use std::sync::Arc;

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
            // Proceed to create a new user
        }
        Err(err) => {
            return HttpResponse::InternalServerError().json(json!({"error": format!("Failed to query user: {}", err)}));
        }
    }
    let role_param = role::id::equals(user.role.clone());
    // Create a new user
    match prisma_client.user().create(
        user.username.clone(),
        user.first_name.clone(),
        user.last_name.clone(),
        user.email.clone(),
        user.password.clone(),
        role_param,
        vec![],
    ).exec().await {
        Ok(new_user) => {
            // Return the created user
            HttpResponse::Created().json(UserResponse {
                id: new_user.id,
                username: new_user.display_name,
                email: new_user.email,
                first_name: new_user.first_name,
                last_name: new_user.last_name,
            })
        }
        Err(err) => {
            HttpResponse::InternalServerError().json(json!({"error": format!("Failed to create user: {}", err)}))
        }
    }
}