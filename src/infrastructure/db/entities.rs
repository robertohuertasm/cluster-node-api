use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::models::{Node, NodeStatus, Operation, OperationType};

#[derive(Debug, Copy, Clone, Deserialize, Serialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "operation_type", rename_all = "lowercase")]
pub enum DbOperationType {
    #[serde(rename = "poweron")]
    PowerOn,
    #[serde(rename = "poweroff")]
    PowerOff,
    #[serde(rename = "reboot")]
    Reboot,
}

impl From<OperationType> for DbOperationType {
    fn from(op_type: OperationType) -> Self {
        match op_type {
            OperationType::PowerOn => DbOperationType::PowerOn,
            OperationType::PowerOff => DbOperationType::PowerOff,
            OperationType::Reboot => DbOperationType::Reboot,
        }
    }
}

impl From<DbOperationType> for OperationType {
    fn from(op_type: DbOperationType) -> Self {
        match op_type {
            DbOperationType::PowerOn => OperationType::PowerOn,
            DbOperationType::PowerOff => OperationType::PowerOff,
            DbOperationType::Reboot => OperationType::Reboot,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq, Eq)]
pub struct DbOperation {
    pub id: Uuid,
    pub node_id: Uuid,
    pub operation_type: DbOperationType,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Operation> for DbOperation {
    fn from(op: Operation) -> Self {
        Self {
            id: op.id,
            node_id: op.node_id,
            operation_type: op.operation_type.into(),
            created_at: op.created_at,
            updated_at: op.updated_at,
        }
    }
}

impl From<DbOperation> for Operation {
    fn from(op: DbOperation) -> Self {
        Self {
            id: op.id,
            node_id: op.node_id,
            operation_type: op.operation_type.into(),
            created_at: op.created_at,
            updated_at: op.updated_at,
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "node_status", rename_all = "lowercase")]
pub enum DbNodeStatus {
    #[serde(rename = "poweron")]
    PowerOn,
    #[serde(rename = "poweroff")]
    PowerOff,
    #[serde(rename = "rebooting")]
    Rebooting,
}

impl From<NodeStatus> for DbNodeStatus {
    fn from(status: NodeStatus) -> Self {
        match status {
            NodeStatus::PowerOn => DbNodeStatus::PowerOn,
            NodeStatus::PowerOff => DbNodeStatus::PowerOff,
            NodeStatus::Rebooting => DbNodeStatus::Rebooting,
        }
    }
}

impl From<DbNodeStatus> for NodeStatus {
    fn from(status: DbNodeStatus) -> Self {
        match status {
            DbNodeStatus::PowerOn => NodeStatus::PowerOn,
            DbNodeStatus::PowerOff => NodeStatus::PowerOff,
            DbNodeStatus::Rebooting => NodeStatus::Rebooting,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq, Eq)]
pub struct DbNode {
    pub id: Uuid,
    pub name: String,
    pub cluster_id: Uuid,
    pub status: DbNodeStatus,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Node> for DbNode {
    fn from(node: Node) -> Self {
        Self {
            id: node.id,
            name: node.name,
            cluster_id: node.cluster_id,
            status: node.status.into(),
            created_at: node.created_at,
            updated_at: node.updated_at,
        }
    }
}

impl From<DbNode> for Node {
    fn from(node: DbNode) -> Self {
        Self {
            id: node.id,
            name: node.name,
            cluster_id: node.cluster_id,
            status: node.status.into(),
            created_at: node.created_at,
            updated_at: node.updated_at,
        }
    }
}
