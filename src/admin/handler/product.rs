use crate::admin::model::*;
use crate::auth::model::Claims;
use crate::prisma::PrismaClient; // Adjust based on your actual imports
use crate::prisma::*;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;


pub async fn create_product(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
    payload: web::Json<ProductPayload>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        if claims.is_admin {
            let category_ids = payload
                .category
                .iter()
                .map(|cat_id| category::id::equals(cat_id.clone()))
                .collect::<Vec<_>>();

            let new_product_result = prisma_client
                .product()
                .create(
                    payload.name.clone(),
                    payload.description.clone(),
                    payload.price,
                    payload.stock,
                    payload.imageurl.clone(),
                    vec![product::categories::connect(category_ids.clone())],
                )
                .exec()
                .await;

            match new_product_result {
                Ok(product) => {
                    let created_product = prisma_client
                        .product()
                        .find_unique(product::id::equals(product.id.clone()))
                        .with(product::categories::fetch(vec![]))
                        .exec()
                        .await;

                    match created_product {
                        Ok(Some(created_product)) => {
                            let response = ProductResponse {
                                id: product.id.clone(),
                                name: product.name.clone(),
                                description: product.description.clone(),
                                price: product.price,
                                stock: product.stock,
                                category: created_product
                                    .categories
                                    .unwrap()
                                    .into_iter()
                                    .map(|cat| cat.name.clone())
                                    .collect::<Vec<String>>(),
                                imageurl: product.image_url.clone(),
                            };
                            HttpResponse::Created().json(response)
                        }
                        Ok(None) => HttpResponse::InternalServerError()
                            .json(json!({"error": "Created product not found."})),
                        Err(_) => HttpResponse::InternalServerError()
                            .json(json!({"error": "Could not fetch created product."})),
                    }
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

pub async fn update_product(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
    product_id: web::Path<String>,
    payload: web::Json<ProductPayload>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        if claims.is_admin {
            let category_ids = payload
                .category
                .iter()
                .map(|cat_id| category::id::equals(cat_id.clone()))
                .collect::<Vec<_>>();

            let update_operations = vec![
                product::name::set(payload.name.clone()),
                product::description::set(payload.description.clone()),
                product::price::set(payload.price),
                product::stock::set(payload.stock),
                product::image_url::set(payload.imageurl.clone()),
                product::categories::connect(category_ids.clone()),
            ];

            let update_product_result = prisma_client
                .product()
                .update(product::id::equals(product_id.clone()), update_operations)
                .exec()
                .await;

            match update_product_result {
                Ok(product) => {
                    // Fetch the updated product along with its categories
                    let updated_product = prisma_client
                        .product()
                        .find_unique(product::id::equals(product_id.clone()))
                        .with(product::categories::fetch(vec![]))
                        .exec()
                        .await;

                    match updated_product {
                        Ok(Some(updated_product)) => {
                            let response = ProductResponse {
                                id: product.id.clone(),
                                name: product.name.clone(),
                                description: product.description.clone(),
                                price: product.price,
                                stock: product.stock,
                                category: updated_product
                                    .categories
                                    .unwrap()
                                    .into_iter()
                                    .map(|cat| cat.name.clone())
                                    .collect::<Vec<String>>(),
                                imageurl: product.image_url.clone(),
                            };
                            HttpResponse::Ok().json(response)
                        }
                        Ok(None) => HttpResponse::InternalServerError()
                            .json(json!({"error": "Updated product not found."})),
                        Err(_) => HttpResponse::InternalServerError()
                            .json(json!({"error": "Could not fetch updated product."})),
                    }
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
