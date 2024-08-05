use super::model::*;
use crate::auth::model::Claims;
use crate::prisma::PrismaClient;
use crate::prisma::*;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

pub async fn place_order(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
    payload: web::Json<PlaceOrderPayload>,
) -> impl Responder {
    let claims = req.extensions().get::<Claims>().unwrap().clone();
    let order_items = payload.productlist.clone();
    let payment_method = payload.paymentmethod.clone();
    let user_id = claims.sub.clone();

    let mut total_price = 0.0;
    let mut create_order_items_query = vec![];
    for item in &order_items {
        if let Ok(Some(product)) = prisma_client
            .product()
            .find_unique(product::id::equals(item.productid.clone()))
            .exec()
            .await
        {
            if product.stock >= item.quantity {
                total_price += product.price * item.quantity as f64;
                create_order_items_query.push(item);
            } else {
                return HttpResponse::BadRequest().json(json!({"error": "Not sufficient Product Stock"}));
            }
        } else {
            return HttpResponse::BadRequest().json(json!({"error": "Invalid product ID."}));
        }
    }

    let order = prisma_client
    .order()
    .create(
        user::id::equals(user_id.clone()),
        "pending".to_string(),
        total_price,
        payment_method.clone(),
        vec![],
    )
    .exec()
    .await;

    match order {
        Ok(order) => {
            // Create the OrderItem records
            let order_items_result = prisma_client
                .order_item()
                .create_many(
                    create_order_items_query
                        .iter()
                        .map(|item| {
                            order_item::create_unchecked(
                                order.id.clone(),
                                item.productid.clone(),
                                item.quantity,
                                vec![],
                            )
                        })
                        .collect(),
                )
                .exec()
                .await;

            match order_items_result {
                Ok(_) => HttpResponse::Ok().json(json!({"message": "Order placed successfully"})),
                Err(err) => {
                    // Handle error and possibly rollback the order creation
                    prisma_client
                        .order()
                        .delete(order::id::equals(order.id.clone()))
                        .exec()
                        .await
                        .ok(); // Ignore the error from rollback
                    HttpResponse::InternalServerError().json(json!({"error": format!("Failed to create order items: {:?}", err)}))
                }
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to create order: {:?}", err)})),
    }
}

pub async fn get_orders(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>(){
        let client_id = claims.sub.clone();
        println!("app passed here successfully, {}", client_id);

        let filter = if claims.is_admin {
            vec![]
        } else {
            vec![order::user_id::equals(client_id.clone())]
        };
        
        match prisma_client
            .order()
            .find_many(filter)
            .with(order::items::fetch(vec![]).with(order_item::product::fetch())) // Nested fetch
            .exec()
            .await
        {
            Ok(orders) => {
                // Map the orders to include product names in the response
                let orders_with_product_names: Vec<_> = orders.into_iter().map(|order| {
                    let items_with_product_names: Vec<_> = match order.items {
                        Some(items) => items.into_iter().map(|item| {
                            let product_name = item.product.unwrap().name; // Assuming product is always present
                            json!({
                                "id": item.id,
                                "productId": item.product_id,
                                "productName": product_name,
                                "quantity": item.quantity,
                                "createdAt": item.created_at,
                                "updatedAt": item.updated_at,
                            })
                        }).collect(),
                        None => vec![],
                    };

                    json!({
                        "id": order.id,
                        "userId": order.user_id,
                        "status": order.status,
                        "payedPrice": order.payed_price,
                        "paymentMethod": order.payment_method,
                        "createdAt": order.created_at,
                        "updatedAt": order.updated_at,
                        "items": items_with_product_names,
                    })
                }).collect();

                HttpResponse::Ok().json(orders_with_product_names)
            },
            Err(err) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to fetch orders: {:?}", err)})),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}
