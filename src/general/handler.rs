use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use crate::prisma::PrismaClient;
use crate::prisma::*;
use std::sync::Arc;
use crate::admin::model::{CategoryResponse, ProductResponse};


pub async fn get_categories(
    prisma_client: web::Data<Arc<PrismaClient>>
) -> impl Responder {
    let categories = prisma_client.category().find_many(vec![]).exec().await;

    match categories {
        Ok(categories) => {
            let response = categories.into_iter().map(|category| {
                CategoryResponse {
                    id: category.id,
                    name: category.name,
                    description: category.description,
                }
            }).collect::<Vec<_>>();
            HttpResponse::Ok().json(response)
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({"error": "Database Error"}))
        },
    }

}
