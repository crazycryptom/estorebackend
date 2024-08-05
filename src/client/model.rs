use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct Product {
    pub productid: String,
    pub quantity: i32,
}

#[derive(Deserialize, Clone)]
pub struct PlaceOrderPayload {
    pub productlist: Vec<Product>,
    pub paymentmethod: String,
}