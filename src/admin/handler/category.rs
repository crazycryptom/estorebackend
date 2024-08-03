use crate::admin::model::*;
use crate::auth::model::{Claims, UserResponse};
use crate::prisma::*;
use crate::{
    prisma::{self, user, PrismaClient},
    RoleType,
}; // Adjust based on your actual imports
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::{any::Any, sync::Arc};

pub async fn create_category(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
    payload: web::Json<CategoryPayload>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        if claims.is_admin {
            let new_category_result = prisma_client
                .category()
                .create(
                    payload.name.clone(),
                    payload.description.clone(),
                    vec![],
                )
                .exec()
                .await;

            match new_category_result {
                Ok(category) => {
                    let response = CategoryResponse {
                        id: category.id.clone(),
                        name: category.name.clone(),
                        description: category.description.clone(),
                    };
                    HttpResponse::Created().json(response)
                }
                Err(_) => HttpResponse::BadRequest().json(json!({"error": "Invalid input data."})),
            }
        } else {
            HttpResponse::Unauthorized().json(json!({"error": "Unauthorized."}))
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized."}))
    }
}

pub async fn update_category(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
    category_id: web::Path<String>,
    payload: web::Json<CategoryPayload>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        if claims.is_admin {
            let category_id = category_id.into_inner();
            let update_result = prisma_client
                .category()
                .update(
                    category::id::equals(category_id.clone()),
                    vec![
                        category::name::set(payload.name.clone()),
                        category::description::set(payload.description.clone()),
                    ],
                )
                .exec()
                .await;

            match update_result {
                Ok(category) => {
                    let response = CategoryResponse {
                        id: category.id.clone(),
                        name: category.name.clone(),
                        description: category.description.clone(),
                    };
                    HttpResponse::Ok().json(response)
                }
                Err(_) => HttpResponse::NotFound().json(json!({"error": "Category not found"})),
            }
        } else {
            HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}

pub async fn delete_category(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
    category_id: web::Path<String>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        if claims.is_admin {
            let category_id = category_id.into_inner();
            let delete_result = prisma_client
                .category()
                .delete(category::id::equals(category_id.clone()))
                .exec()
                .await;

            match delete_result {
                Ok(_) => HttpResponse::Ok().json(json!({"message": "Category deleted successfully"})),
                Err(_) => HttpResponse::NotFound().json(json!({"error": "Category not found"})),
            }
        } else {
            HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}