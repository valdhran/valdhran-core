use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::value_objects::email::Email;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: Email,
    pub display_name: String,
    pub password_hash: String,      // Argon2 hash — nunca el password en claro
    pub role_ids: Vec<Uuid>,        // roles asignados al usuario
    pub status: UserStatus,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        tenant_id: Uuid,
        email: Email,
        display_name: impl Into<String>,
        password_hash: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            email,
            display_name: display_name.into(),
            password_hash: password_hash.into(),
            role_ids: Vec::new(),
            status: UserStatus::Active,
            last_login_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }

    pub fn assign_role(&mut self, role_id: Uuid) {
        if !self.role_ids.contains(&role_id) {
            self.role_ids.push(role_id);
            self.updated_at = Utc::now();
        }
    }
}
