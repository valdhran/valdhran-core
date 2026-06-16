use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Permiso atómico del sistema.
/// Ej: "invoices:create", "payroll:read", "users:delete"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: Uuid,
    pub resource: String,   // ej: "invoices", "payroll", "users"
    pub action: String,     // ej: "create", "read", "update", "delete"
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl Permission {
    pub fn new(resource: impl Into<String>, action: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            resource: resource.into(),
            action: action.into(),
            description: description.into(),
            created_at: Utc::now(),
        }
    }

    /// Nombre canónico del permiso: "resource:action"
    pub fn canonical_name(&self) -> String {
        format!("{}:{}", self.resource, self.action)
    }
}
