use crate::admin::model::{CategoryResponse, GetProductsPagniationQuery, ProductResponse};
use crate::{prisma::PrismaClient, product, order};
use crate::auth::model::Claims;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

pub async fn get_categories(prisma_client: web::Data<Arc<PrismaClient>>) -> impl Responder {
    let categories = prisma_client.category().find_many(vec![]).exec().await;

    match categories {
        Ok(categories) => {
            let response = categories
                .into_iter()
                .map(|category| CategoryResponse {
                    id: category.id,
                    name: category.name,
                    description: category.description,
                })
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": "Database Error"})),
    }
}

pub async fn get_products(
    prisma_client: web::Data<Arc<PrismaClient>>,
    query: web::Query<GetProductsPagniationQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let category = query.category.clone();
    let id = query.id.clone();

    let mut filter_conditions = vec![];
    if let Some(id) = id {
        filter_conditions.push(product::id::equals(id));
    }
    if let Some(category_id) = category {
        filter_conditions.push(product::categories::some(vec![
            crate::prisma::category::id::equals(category_id),
        ]));
    }

    let total_items = prisma_client
        .product()
        .count(filter_conditions.clone())
        .exec()
        .await
        .unwrap_or(0);
    let total_pages = (total_items as f64 / limit as f64).ceil() as i64;

    let offset = (page - 1) * limit;

    let products = prisma_client
        .product()
        .find_many(filter_conditions)
        .with(product::categories::fetch(vec![]))
        .skip(offset)
        .take(limit)
        .exec()
        .await
        .unwrap_or(vec![]);
    let product_response = products
        .into_iter()
        .map(|product| ProductResponse {
            id: product.id,
            name: product.name,
            description: product.description,
            price: product.price,
            stock: product.stock,
            imageurl: product.image_url,
            category: product
            .categories
            .map_or(vec![], |cats| {
                cats.into_iter()
                    .map(|cat| cat.name.clone())
                    .collect::<Vec<String>>()
            }),
        })
        .collect::<Vec<_>>();
    let response = json!({
        "products": product_response,
        "pagination": {
            "currentPage": page,
            "totalPages": total_pages,
            "totalItems": total_items,
            "limit": limit,
        }
    });
    HttpResponse::Ok().json(response)
}

// pub async fn get_orders(
//     req: HttpRequest,
//     prisma_client: web::Data<Arc<PrismaClient>>,
//     client_id: web::Path<String>,
// ) -> impl Responder {
//     if let Some(claims) = req.extensions().get::<Claims>() {
//         let filter = if claims.is_admin {
//             // Admin case: filter by client_id if provided
//             if !client_id.is_empty() {
//                 vec![order::user_id::equals(client_id.clone())]
//             } else {
//                 vec![]
//             }
//         } else {
//             // Regular user case: filter by the user's own ID
//             vec![order::user_id::equals(claims.sub.clone())]
//         };
//         match prisma_client
//             .order()
//             .find_many(filter)
//             .exec()
//             .await
//         {
//             Ok(orders) => HttpResponse::Ok().json(orders),
//             Err(err) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to fetch orders: {:?}", err)})),
//         }
//     } else {
//         HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
//     }
// }
