use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::entities::permission::Permission;

/// Rol dentro de un tenant.
/// El RBAC es jerárquico: un rol puede tener un rol padre que hereda sus permisos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: String,
    pub parent_role_id: Option<Uuid>,   // herencia jerárquica
    pub permissions: Vec<Permission>,
    pub is_system_role: bool,           // roles del sistema no se pueden eliminar
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Role {
    pub fn new(
        tenant_id: Uuid,
        name: impl Into<String>,
        description: impl Into<String>,
        parent_role_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            name: name.into(),
            description: description.into(),
            parent_role_id,
            permissions: Vec::new(),
            is_system_role: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn has_permission(&self, resource: &str, action: &str) -> bool {
        self.permissions.iter().any(|p| p.resource == resource && p.action == action)
    }

    pub fn add_permission(&mut self, permission: Permission) {
        if !self.has_permission(&permission.resource, &permission.action) {
            self.permissions.push(permission);
            self.updated_at = Utc::now();
        }
    }
}
