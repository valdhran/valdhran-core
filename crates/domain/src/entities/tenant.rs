use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::value_objects::tenant_slug::TenantSlug;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantStatus {
    Active,
    Suspended,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub slug: TenantSlug,         // identificador único legible (ej: "empresa-peru-sac")
    pub name: String,             // nombre legal de la empresa
    pub schema_name: String,      // nombre del schema PostgreSQL (ej: "tenant_empresa_peru_sac")
    pub status: TenantStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tenant {
    pub fn new(slug: TenantSlug, name: impl Into<String>) -> Self {
        let schema_name = format!("tenant_{}", slug.value().replace('-', "_"));
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            schema_name,
            name: name.into(),
            slug,
            status: TenantStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == TenantStatus::Active
    }
}
