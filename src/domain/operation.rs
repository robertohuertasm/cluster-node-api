use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Copy, Clone, Deserialize, Serialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "node_status", rename_all = "lowercase")]
pub enum OperationType {
    #[serde(rename = "poweron")]
    PowerOn,
    #[serde(rename = "poweroff")]
    PowerOff,
    #[serde(rename = "reboot")]
    Reboot,
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq, Eq)]
pub struct Operation {
    pub id: Uuid,
    pub operation_type: OperationType,
    pub node_id: Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
