use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use valdhran_domain::{
    entities::role::Role,
    errors::{DomainError, DomainResult},
    repositories::role_repository::RoleRepository,
};

pub struct PgRoleRepository {
    pool: PgPool,
    schema: String,
}

impl PgRoleRepository {
    pub fn new(pool: PgPool, schema: String) -> Self {
        Self { pool, schema }
    }
}

#[async_trait]
impl RoleRepository for PgRoleRepository {
    async fn find_by_id(&self, tenant_id: Uuid, role_id: Uuid) -> DomainResult<Option<Role>> {
        let query = format!(
            "SELECT id, name, description, parent_role_id, is_system_role, created_at, updated_at
             FROM {}.roles WHERE id = $1",
            self.schema
        );
        let row = sqlx::query_as::<_, RoleRow>(&query)
            .bind(role_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(row.map(|r| r.into_role(tenant_id)))
    }

    async fn find_all_by_tenant(&self, tenant_id: Uuid) -> DomainResult<Vec<Role>> {
        let query = format!(
            "SELECT id, name, description, parent_role_id, is_system_role, created_at, updated_at
             FROM {}.roles ORDER BY name",
            self.schema
        );
        let rows = sqlx::query_as::<_, RoleRow>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_role(tenant_id)).collect())
    }

    async fn save(&self, role: &Role) -> DomainResult<()> {
        let query = format!(
            r#"INSERT INTO {schema}.roles (id, name, description, parent_role_id, is_system_role, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (id) DO UPDATE SET
                   name           = EXCLUDED.name,
                   description    = EXCLUDED.description,
                   parent_role_id = EXCLUDED.parent_role_id,
                   updated_at     = EXCLUDED.updated_at"#,
            schema = self.schema
        );
        sqlx::query(&query)
            .bind(role.id)
            .bind(&role.name)
            .bind(&role.description)
            .bind(role.parent_role_id)
            .bind(role.is_system_role)
            .bind(role.created_at)
            .bind(role.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, _tenant_id: Uuid, role_id: Uuid) -> DomainResult<()> {
        let query = format!(
            "DELETE FROM {}.roles WHERE id = $1 AND is_system_role = FALSE",
            self.schema
        );
        sqlx::query(&query)
            .bind(role_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct RoleRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    parent_role_id: Option<Uuid>,
    is_system_role: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl RoleRow {
    fn into_role(self, tenant_id: Uuid) -> Role {
        Role {
            id: self.id,
            tenant_id,
            name: self.name,
            description: self.description.unwrap_or_default(),
            parent_role_id: self.parent_role_id,
            permissions: vec![],
            is_system_role: self.is_system_role,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
