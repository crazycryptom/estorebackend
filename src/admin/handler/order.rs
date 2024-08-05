use crate::auth::model::Claims;
use crate::prisma::PrismaClient; // Adjust based on your actual imports
use crate::prisma::*;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

pub async fn approve_order(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
    order_id: web::Path<String>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        // Verify user authorization, assuming only admins can approve orders
        if !claims.is_admin {
            return HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        }

        let order_id = order_id.into_inner();

        // Fetch the order to check its current status
        match prisma_client
            .order()
            .find_unique(order::id::equals(order_id.clone()))
            .with(order::items::fetch(vec![]).with(order_item::product::fetch()))
            .exec()
            .await
        {
            Ok(Some(order)) => {
                // Check if the order status is "pending"
                if order.status == "pending" {
                    // Start a transaction to update order status and reduce product stocks
                    let transaction_result: Result<(), prisma_client_rust::QueryError> = prisma_client._transaction().run(|client| {
                        Box::pin(async move {
                            // Update order status to "approved"
                            client.order()
                                .update(
                                    order::id::equals(order_id.clone()),
                                    vec![order::status::set("approved".to_string())]
                                )
                                .exec()
                                .await
                                .map_err(|e| e)?;

                            // Reduce product stocks based on order items
                            for item in order.items.unwrap() {
                                let product_id = item.product_id.clone();
                                let new_stock = item.product.unwrap().stock - item.quantity;

                                client.product()
                                    .update(
                                        product::id::equals(product_id),
                                        vec![product::stock::set(new_stock)]
                                    )
                                    .exec()
                                    .await
                                    .map_err(|e| e)?;
                            }

                            Ok(())
                        })
                    }).await;

                    match transaction_result {
                        Ok(_) => HttpResponse::Ok().json(json!({
                            "message": "Order approved successfully",
                        })),
                        Err(err) => HttpResponse::InternalServerError().json(json!({
                            "error": format!("Failed to approve order and reduce product stocks: {:?}", err)
                        })),
                    }
                } else {
                    HttpResponse::BadRequest().json(json!({"error": "Order is not in pending status"}))
                }
            },
            Ok(None) => HttpResponse::NotFound().json(json!({"error": "Order not found"})),
            Err(err) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to fetch order: {:?}", err)
            })),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}