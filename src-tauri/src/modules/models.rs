use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveServerProfileInput {
    pub name: Option<String>,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerProfileSummary {
    pub id: String,
    pub name: Option<String>,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub created_at: i64,
    pub updated_at: i64,
}
