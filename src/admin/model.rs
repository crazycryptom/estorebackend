use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: Option<i64>,
    pub search: Option<String>,
}
#[derive(Deserialize)]
pub struct RolePayload {
    pub role: String,
}

#[derive(Deserialize)]
pub struct ProductPayload {
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: i32,
    pub category: Vec<String>,
    pub imageurl: String,
}

#[derive(Serialize)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: i32,
    pub category: Vec<String>,
    pub imageurl: String,
}

#[derive(Deserialize)]
pub struct CategoryPayload{
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub description: String,
}

fn default_page() -> Option<i64> {
    Some(1)
}

fn default_limit() -> Option<i64> {
    Some(10)
}
