use actix_web::{dev::Response, web::{self, Json}, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use crate::{prisma::{self, user, PrismaClient}, RoleType}; // Adjust based on your actual imports
use std::{any::Any, sync::Arc};
use crate::auth::model::{ Claims, UserResponse };
use super::model::*;

pub async fn get_users(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        println!("the app passed here");
        match prisma_client
            .user()
            .find_unique(user::id::equals(claims.sub.clone()))
            .exec()
            .await 
        {
            Ok(Some(user)) => {
                if user.role == RoleType::Admin {
                    let page = query.page.unwrap_or(1);
                    let limit = query.limit.unwrap_or(10);
                    let search = query.search.as_deref().unwrap_or("");

                    // Fetch users with optional search
                    let total_items = prisma_client
                        .user()
                        .count(vec![])
                        .exec()
                        .await
                        .unwrap_or(0);

                    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;
                    println!("authorized user call get user request");

                    let users = prisma_client
                        .user()
                        .find_many(
                            vec![
                                user::display_name::contains(search.to_string()),
                                user::email::contains(search.to_string())
                                ]
                        )
                        .take(limit)
                        .skip((page - 1) * limit as i64)
                        .exec()
                        .await
                        .unwrap_or_default();

                    let response = json!({
                        "users": users.iter().map(|u| {
                            json!({
                                "id": u.id,
                                "username": u.display_name,
                                "email": u.email,
                                "firstName": u.first_name,
                                "lastName": u.last_name,
                                "role": u.role,
                                "createdAt": u.created_at,
                            })
                        }).collect::<Vec<_>>(),
                        "pagination": {
                            "currentPage": page,
                            "totalPages": total_pages,
                            "totalItems": total_items,
                            "limit": limit,
                        }
                    });

                    HttpResponse::Ok().json(response)
                } else {
                    HttpResponse::Unauthorized().json(json!({"error": "You don't have admin privilege."}))
                }
            },
            Ok(None) => {
                HttpResponse::NotFound().json(json!({"error": "User not found."}))
            },
            Err(_) => {
                HttpResponse::InternalServerError().json(json!({"error": "Database error."}))
            }
        }
    } else {
        println!("the app passed here");
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized."}))
    }
}

pub async fn update_user_role(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
    user_id: web::Path<String>,
    payload: web::Json<RolePayload>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        if claims.is_admin {
            let updated_user = prisma_client.user().find_unique(
                user::id::equals(user_id.clone())
            ).exec().await;

            match updated_user {
                Ok(Some(user)) => {
                    let response = UserResponse {
                        id: user.id,
                        username: user.display_name,
                        email: user.email,
                        first_name: user.first_name,
                        last_name: user.last_name,
                        role: match user.role {
                            RoleType::Admin => String::from("admin"),
                            _ => String::from("client"),
                        },
                    };
                    HttpResponse::Ok().json(response)
                },
                Ok(None) => {
                    HttpResponse::NotFound().json(json!({"error": "User not found"}))
                },
                Err(_) => {
                    HttpResponse::InternalServerError().json(json!({"error": "database error"}))
                }
            }
        } else {
            HttpResponse::Unauthorized().json(json!({"error": "Unauthorized."}))
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized."}))
    }
}
