use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum OperationType {
    #[serde(rename = "poweron")]
    PowerOn,
    #[serde(rename = "poweroff")]
    PowerOff,
    #[serde(rename = "reboot")]
    Reboot,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Operation {
    pub id: Uuid,
    pub node_id: Uuid,
    pub operation_type: OperationType,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Operation {
    pub fn new(node_id: Uuid, operation_type: OperationType) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_id,
            operation_type,
            created_at: None,
            updated_at: None,
        }
    }
}
