use crate::admin::model::*;
use crate::auth::model::Claims;
use crate::prisma::PrismaClient;
use crate::prisma::*;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};
use serde_json::json;
use std::sync::Arc;

pub async fn sales_result(
    req: HttpRequest,
    prisma_client: web::Data<Arc<PrismaClient>>,
    query: web::Query<SalesQuery>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        if !claims.is_admin {
            return HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        }

        let start_date = query
            .start_date
            .as_ref()
            .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

        let end_date = query
            .end_date
            .as_ref()
            .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
            .unwrap_or_else(|| Utc::now().date_naive());

        let start_datetime: DateTime<FixedOffset> = FixedOffset::east_opt(0)
            .unwrap()
            .from_local_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap())
            .unwrap();
        let end_datetime: DateTime<FixedOffset> = FixedOffset::east_opt(0)
            .unwrap()
            .from_local_datetime(&end_date.and_hms_opt(23, 59, 59).unwrap())
            .unwrap();
        match prisma_client
            .order()
            .find_many(vec![
                order::status::equals("approved".to_string()),
                order::created_at::gte(start_datetime),
                order::created_at::lte(end_datetime),
            ])
            .with(order::items::fetch(vec![]).with(order_item::product::fetch())) // Nested fetch for items and products
            .exec()
            .await
        {
            Ok(orders) => {
                let mut product_sales: std::collections::HashMap<String, f64> =
                    std::collections::HashMap::new();

                for order in orders {
                    if let Some(items) = order.items {
                        for item in items {
                            let product_id = item.product_id.clone();
                            let quantity = item.quantity as f64;

                            let entry = product_sales.entry(product_id).or_insert(0.0);
                            *entry += quantity;
                        }
                    }
                }

                let mut sales_response = vec![];

                for (product_id, sales_amount) in product_sales {
                    if let Ok(Some(product)) = prisma_client
                        .product()
                        .find_unique(product::id::equals(product_id.clone()))
                        .with(product::categories::fetch(vec![]))
                        .exec()
                        .await
                    {
                        sales_response.push(json!({
                            "id": product.id,
                            "name": product.name,
                            "description": product.description,
                            "price": product.price,
                            "imgUrl": product.image_url,
                            "salesAmount": sales_amount,
                            "category": product.categories
                            .unwrap()
                            .into_iter()
                            .map(|cat| cat.name.clone())
                            .collect::<Vec<String>>(),
                        }));
                    }
                }

                HttpResponse::Ok().json(sales_response)
            }
            Err(err) => HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to fetch orders: {:?}", err)
            })),
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}))
    }
}
