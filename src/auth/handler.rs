use crate::auth::model::{
    Claims, GetRecoveryKeyPayload, LoginUser, Passwords, RegisterUser, ResetPasswordPayload,
    UpdateProfile, UserResponse,
};
use crate::prisma::*;
use crate::prisma::{self, PrismaClient};
use crate::utils::get_secret_key;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use base32;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::Rng;
use serde_json::json;
use std::sync::Arc;
use totp_rs::{Algorithm, Secret, TOTP};

use super::model::VerifyOTPSchema;

pub async fn register_user(
    user: web::Json<RegisterUser>,
    prisma_client: web::Data<Arc<PrismaClient>>,
) -> impl Responder {
    match prisma_client
        .user()
        .find_unique(user::email::equals(user.email.clone()))
        .exec()
        .await
    {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().json(json!({"error": "User already exists"}));
        }
        Ok(None) => {
            let hashed_password = match hash(user.password.clone(), DEFAULT_COST) {
                Ok(p) => p,
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Failed to hash password"}))
                }
            };

            let role_type = match user.role.as_str() {
                "admin" => RoleType::Admin,
                _ => RoleType::Client,
            };

            match prisma_client
                .user()
                .create(
                    user.username.clone(),
                    user.first_name.clone(),
                    user.last_name.clone(),
                    user.email.clone(),
                    hashed_password.clone(),
                    role_type.clone(),
                    vec![],
                )
                .exec()
                .await
            {
                Ok(new_user) => {
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
                        otp_enabled: new_user.otp_enabled,
                        otp_verified: new_user.otp_verified,
                        otp_auth_url: new_user.otp_auth_url.to_owned(),
                        otp_base32: new_user.opt_base_32.to_owned(),
                    });
                }
                Err(err) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": format!("Failed to create user: {}", err)}));
                }
            };
        }
        Err(err) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Failed to query user: {}", err)}));
        }
    }
}

pub async fn login_user(
    user: web::Json<LoginUser>,
    prisma_client: web::Data<Arc<PrismaClient>>,
) -> impl Responder {
    match prisma_client
        .user()
        .find_unique(user::email::equals(user.email.clone()))
        .exec()
        .await
    {
        Ok(Some(user_record)) => {
            println!(
                "user record is {} and password is {}",
                user_record.password, user.password
            );
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
                            is_admin: user_record.role == RoleType::Admin,
                        };

                        let secret = get_secret_key();
                        let token = match encode(
                            &Header::default(),
                            &claims,
                            &EncodingKey::from_secret(secret.as_bytes()),
                        ) {
                            Ok(t) => t,
                            Err(_) => {
                                return HttpResponse::InternalServerError()
                                    .json(json!({"error": "Failed to generate token"}))
                            }
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
                                otp_enabled: user_record.otp_enabled,
                                otp_verified: user_record.otp_verified,
                                otp_auth_url: user_record.otp_auth_url.to_owned(),
                                otp_base32: user_record.opt_base_32.to_owned(),

                            }
                        }));
                    } else {
                        return HttpResponse::Unauthorized()
                            .json(json!({"error": "Invalid credentials"}));
                    }
                }
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "invalid email or password"}));
                }
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"})),
        Err(err) => HttpResponse::InternalServerError()
            .json(json!({"error": format!("Failed to query user: {}", err)})),
    }
}

pub async fn change_pass(
    req: actix_web::HttpRequest,
    passwords: web::Json<Passwords>,
    prisma_client: web::Data<Arc<PrismaClient>>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        match prisma_client
            .user()
            .update(
                user::id::equals(claims.sub.clone()),
                vec![user::password::set(passwords.newpassword.clone())],
            )
            .exec()
            .await
        {
            Ok(_) => return HttpResponse::Ok().json("password is updated successfully"),
            Err(_) => return HttpResponse::NotFound().json("Didn't find user"),
        }
    }
    return HttpResponse::Ok().json(
        json!({"oldpassword": passwords.oldpassword, "newPassword": passwords.newpassword }),
    );
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
            Ok(Some(_)) => {
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
                        otp_enabled: updated_user.otp_enabled,
                        otp_verified: updated_user.otp_verified,
                        otp_auth_url: updated_user.otp_auth_url.to_owned(),
                        otp_base32: updated_user.opt_base_32.to_owned(),
                    }),
                    Err(_) => {
                        HttpResponse::BadRequest().json(json!({"error": "Invalid input data"}))
                    }
                }
            }
            Ok(None) => HttpResponse::BadRequest().json(json!({"error": "Invalid input data"})),
            Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Database error"})),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}

pub async fn recovery_key(
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
            Ok(Some(_)) => {
                match prisma_client
                    .user()
                    .update(
                        user::email::equals(payload.email.clone()),
                        vec![user::key::set(Some(recoverykey))],
                    )
                    .exec()
                    .await
                {
                    Ok(updated_user) => {
                        return HttpResponse::Ok().json(json!({"recoveryKey": updated_user.key}))
                    }
                    Err(_) => HttpResponse::BadRequest().json(json!({"error": "Invalid email"})),
                }
            }
            Ok(None) => HttpResponse::BadRequest().json(json!({"error": "Invalid input data"})),
            Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Database error"})),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}

pub async fn reset_password(
    payload: web::Json<ResetPasswordPayload>,
    prisma_client: web::Data<Arc<PrismaClient>>,
) -> impl Responder {
    match prisma_client
        .user()
        .find_first(vec![
            user::key::equals(Some(payload.recoverykey.clone())),
            user::email::equals(payload.email.clone()),
        ])
        .exec()
        .await
    {
        Ok(Some(_)) => {
            let hashed_password = match hash(payload.newpassword.clone(), DEFAULT_COST) {
                Ok(p) => p,
                Err(_) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({"error": "Failed to hash password"}))
                }
            };
            match prisma_client
                .user()
                .update(
                    user::email::equals(payload.email.clone()),
                    vec![user::password::set(hashed_password.clone())],
                )
                .exec()
                .await
            {
                Ok(_) => HttpResponse::Ok().json(json!({"message": "Password reset successfully"})),
                Err(_) => HttpResponse::BadRequest().json(json!({"error": "Invalid input data"})),
            }
        }
        Ok(None) => HttpResponse::BadRequest().json(json!({"error": "Invalid input data"})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Database error"})),
    }
}

pub async fn generate_otp(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        let mut rng = rand::thread_rng();
        let data_byte: [u8; 21] = rng.gen();
        let base32_string =
            base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &data_byte);
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(base32_string).to_bytes().unwrap(),
        )
        .unwrap();
        let otp_base32 = totp.get_secret_base32();
        let issuer = "CodevoWeb";
        let email = prisma_client
            .user()
            .find_unique(user::id::equals(claims.sub.clone()))
            .exec()
            .await
            .unwrap()
            .unwrap()
            .email;

        let otp_auth_url =
            format!("otpauth://totp/{issuer}:{email}?secret={otp_base32}&issuer={issuer}");
        let result = prisma_client
            .user()
            .update(
                user::id::equals(claims.sub.clone()),
                vec![
                    user::opt_base_32::set(Some(otp_base32.clone())),
                    user::otp_auth_url::set(Some(otp_auth_url.clone())),
                ],
            )
            .exec()
            .await;
        match result {
            Ok(_) => HttpResponse::Ok().json(
                json!({"base32":otp_base32.to_owned(), "otpauth_url": otp_auth_url.to_owned()}),
            ),
            Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Database error"})),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}

pub async fn verify_otp(
    req: HttpRequest,
    body: web::Json<VerifyOTPSchema>,
    prisma_client: web::Data<Arc<PrismaClient>>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        let user = prisma_client
            .user()
            .find_unique(user::id::equals(claims.sub.clone()))
            .exec()
            .await
            .unwrap()
            .unwrap();

        let otp_base32 = user.opt_base_32.clone().unwrap();

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(otp_base32).to_bytes().unwrap(),
        )
        .unwrap();

        let is_valid = totp.check_current(body.token.as_ref()).unwrap();

        if !is_valid {
            let json_error = json! ({
                "status": "fail".to_string(),
                "message": "Token is invalid or user doesn't exist".to_string(),
            });

            return HttpResponse::Forbidden().json(json_error);
        }

        match prisma_client
            .user()
            .update(
                user::id::equals(claims.sub.clone()),
                vec![user::otp_enabled::set(true), user::otp_verified::set(true)],
            )
            .exec()
            .await
        {
            Ok(_) => HttpResponse::Ok().json(json!({"otp_verified": true, "user": user})),
            Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Database error"})),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}

pub async fn validate_otp(
    req: HttpRequest,
    body: web::Json<VerifyOTPSchema>,
    prisma_client: web::Data<Arc<PrismaClient>>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        let user = prisma_client
            .user()
            .find_unique(user::id::equals(claims.sub.clone()))
            .exec()
            .await
            .unwrap()
            .unwrap();

        if !user.otp_enabled {
            let json_error = json! ({
                "status": "fail".to_string(),
                "message": "2FA not enabled".to_string(),
            });

            return HttpResponse::Forbidden().json(json_error);
        }

        let otp_base32 = user.opt_base_32.clone().unwrap();

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(otp_base32).to_bytes().unwrap(),
        )
        .unwrap();

        let is_valid = totp.check_current(body.token.as_ref()).unwrap();

        if !is_valid {
            let json_error = json! ({
                "status": "fail".to_string(),
                "message": "Token is invalid or user doesn't exist".to_string(),
            });

            return HttpResponse::Forbidden().json(json_error);
        }
        HttpResponse::Ok().json(json!({"otp_valid": true}))
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}

pub async fn disable_otp(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        let result = prisma_client
        .user()
        .update(
            user::id::equals(claims.sub.clone()),
            vec![
                user::otp_auth_url::set(None),
                user::opt_base_32::set(None),
                user::otp_enabled::set(false),
                user::otp_verified::set(false)
            ]
        )
        .exec()
        .await;

        match result {
            Ok(user) => HttpResponse::Ok().json(json!({"user": user})),
            Err(_) => HttpResponse::InternalServerError().json(json!({"error":"database error"}))
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "the user is unauthorized"}))
    }
}
