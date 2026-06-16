use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use valdhran_domain::{
    entities::tenant::{Tenant, TenantStatus},
    errors::{DomainError, DomainResult},
    repositories::tenant_repository::TenantRepository,
    value_objects::tenant_slug::TenantSlug,
};

pub struct PgTenantRepository {
    pool: PgPool,
}

impl PgTenantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TenantRow {
    id: Uuid,
    slug: String,
    name: String,
    schema_name: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TenantRow {
    fn into_tenant(self) -> Tenant {
        Tenant {
            id: self.id,
            slug: TenantSlug::new(&self.slug).expect("Invalid slug in DB"),
            name: self.name,
            schema_name: self.schema_name,
            status: match self.status.as_str() {
                "suspended" => TenantStatus::Suspended,
                "cancelled" => TenantStatus::Cancelled,
                _ => TenantStatus::Active,
            },
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[async_trait]
impl TenantRepository for PgTenantRepository {
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Tenant>> {
        let row = sqlx::query_as::<_, TenantRow>(
            "SELECT id, slug, name, schema_name, status, created_at, updated_at FROM public.tenants WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| DomainError::Validation(e.to_string()))?;

        Ok(row.map(|r| r.into_tenant()))
    }

    async fn find_by_slug(&self, slug: &TenantSlug) -> DomainResult<Option<Tenant>> {
        let row = sqlx::query_as::<_, TenantRow>(
            "SELECT id, slug, name, schema_name, status, created_at, updated_at FROM public.tenants WHERE slug = $1"
        )
        .bind(slug.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e: sqlx::Error| DomainError::Validation(e.to_string()))?;

        Ok(row.map(|r| r.into_tenant()))
    }

    async fn save(&self, tenant: &Tenant) -> DomainResult<()> {
        let status = match tenant.status {
            TenantStatus::Active    => "active",
            TenantStatus::Suspended => "suspended",
            TenantStatus::Cancelled => "cancelled",
        };

        sqlx::query(
            r#"INSERT INTO public.tenants (id, slug, name, schema_name, status, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (id) DO UPDATE SET
                   name       = EXCLUDED.name,
                   status     = EXCLUDED.status,
                   updated_at = EXCLUDED.updated_at"#
        )
        .bind(tenant.id)
        .bind(tenant.slug.value())
        .bind(&tenant.name)
        .bind(&tenant.schema_name)
        .bind(status)
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e: sqlx::Error| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    async fn exists_by_slug(&self, slug: &TenantSlug) -> DomainResult<bool> {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM public.tenants WHERE slug = $1)"
        )
        .bind(slug.value())
        .fetch_one(&self.pool)
        .await
        .map_err(|e: sqlx::Error| DomainError::Validation(e.to_string()))?;

        Ok(row.0)
    }
}
