use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Copy, Clone, Deserialize, Serialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "node_status", rename_all = "lowercase")]
pub enum NodeStatus {
    #[serde(rename = "poweron")]
    PowerOn,
    #[serde(rename = "poweroff")]
    PowerOff,
    #[serde(rename = "rebooting")]
    Rebooting,
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq, Eq)]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub cluster_id: Uuid,
    pub status: NodeStatus,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
