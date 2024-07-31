use actix_web::dev::Payload;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use crate::prisma::PrismaClient;
use crate::prisma::*;
use crate::auth::model::{RegisterUser, UserResponse, LoginUser,Passwords, Claims, UpdateProfile, GetRecoveryKeyPayload, ResetPasswordPayload};
use super::utils::get_secret_key;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
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
                        role: match new_user.role {
                            RoleType::Admin => String::from("admin"),
                            _ => String::from("client"),
                        },
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
            println!("user record is {} and password is {}",user_record.password, user.password);
            match verify(user.password.clone(), &user_record.password) {
                Ok(valid) => {
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
                        let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())) {
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
                                role: match user_record.role {
                                    RoleType::Admin => String::from("admin"),
                                    _ => String::from("client"),
                                },
                            }
                        }));
                    } else {
                        return HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"}));
                    }
                }
                Err(_) => {
                    // Log this error for debugging purposes
                    return HttpResponse::InternalServerError().json(json!({"error": "invalid email or password"}));
                }
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"})),
        Err(err) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to query user: {}", err)})),
    }
}

pub async fn change_pass (
    req: actix_web::HttpRequest,
    passwords: web::Json<Passwords>,
    prisma_client: web::Data<Arc<PrismaClient>>
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        match prisma_client.user().update(
            user::id::equals(claims.sub.clone()),
            vec![user::password::set(passwords.newpassword.clone())]
        )
        .exec()
        .await {
            Ok(_) => return HttpResponse::Ok().json("password is updated successfully"),
            Err(_) => return HttpResponse::NotFound().json("Didn't find user"),
        }
    }
    return HttpResponse::Ok().json(json!({"oldpassword": passwords.oldpassword, "newPassword": passwords.newpassword }));
}

pub async fn update_profile(
    req: HttpRequest,
    newprofile: web::Json<UpdateProfile>,
    prisma_client: web::Data<Arc<PrismaClient>>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        match prisma_client
            .user()
            .find_unique(user::id::equals(claims.sub.clone()))
            .exec()
            .await
        {
            Ok(Some(user)) => {
                match prisma_client
                    .user()
                    .update(
                        user::id::equals(claims.sub.clone()),
                        vec![
                            user::display_name::set(newprofile.username.clone()),
                            user::first_name::set(newprofile.firstname.clone()),
                            user::last_name::set(newprofile.lastname.clone()),
                            user::email::set(newprofile.email.clone()),
                        ],
                    )
                    .exec()
                    .await
                {
                    Ok(updated_user) => HttpResponse::Ok().json(UserResponse {
                        id: updated_user.id,
                        username: updated_user.display_name,
                        email: updated_user.email,
                        first_name: updated_user.first_name,
                        last_name: updated_user.last_name,
                        role: match updated_user.role {
                            RoleType::Admin => String::from("admin"),
                            _ => String::from("client"),
                        },
                    }),
                    Err(_) => HttpResponse::BadRequest().json(json!({"error": "Invalid input data"})),
                }
            }
            Ok(None) => HttpResponse::BadRequest().json(json!({"error": "Invalid input data"})),
            Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Database error"})),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}

pub async fn recovery_key (
    req: HttpRequest,
    payload: web::Json<GetRecoveryKeyPayload>,
    prisma_client: web::Data<Arc<PrismaClient>>,
) -> impl Responder {
    let recoverykey = "123456".to_string();
    if let Some(claims) = req.extensions().get::<Claims>() {
        match prisma_client
            .user()
            .find_unique(user::id::equals(claims.sub.clone()))
            .exec()
            .await
        {
            Ok(Some(user)) => {
                match prisma_client.user().update(
                    user::email::equals(payload.email.clone()),
                    vec![user::key::set(Some(recoverykey))]
                )
                .exec()
                .await
                {
                    Ok(updated_user) => return HttpResponse::Ok().json(json!({"recoveryKey": updated_user.key})),
                    Err(_) => HttpResponse::BadRequest().json(json!({"error": "Invalid email"}))
                }
            },
            Ok(None) => HttpResponse::BadRequest().json(json!({"error": "Invalid input data"})),
            Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Database error"})),
        }

    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}

pub async fn reset_password (
    payload: web::Json<ResetPasswordPayload>,
    prisma_client: web::Data<Arc<PrismaClient>>
) -> impl Responder {
    match prisma_client
        .user()
        .find_first(vec![
            user::key::equals(Some(payload.recoverykey.clone())),
            user::email::equals(payload.email.clone())
            ])
        .exec()
        .await
    {
        Ok(Some(user)) => {
            let hashed_password = match hash(payload.newpassword.clone(), DEFAULT_COST) {
                Ok(p) => p,
                Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Failed to hash password"})),
            };
            match prisma_client.user().update(
                user::email::equals(payload.email.clone()),
                vec![user::password::set(hashed_password.clone())]
            )
            .exec()
            .await
            {
                Ok(updated_user) => HttpResponse::Ok().json(json!({"message": "Password reset successfully"})),
                Err(_) => HttpResponse::BadRequest().json(json!({"error": "Invalid input data"}))
            }
        },
        Ok(None) => HttpResponse::BadRequest().json(json!({"error": "Invalid input data"})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Database error"})),
    }
}
