use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: Option<i64>,
    #[serde(default = "default_limit")]
    pub limit: Option<i64>,
    pub search: Option<String>,
}

fn default_page() -> Option<i64> {
    Some(1)
}

fn default_limit() -> Option<i64> {
    Some(10)
}
