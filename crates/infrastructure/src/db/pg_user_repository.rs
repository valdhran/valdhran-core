use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use valdhran_domain::{
    entities::user::{User, UserStatus},
    errors::{DomainError, DomainResult},
    repositories::user_repository::UserRepository,
    value_objects::email::Email,
};

pub struct PgUserRepository {
    pool: PgPool,
    schema: String, // schema del tenant, ej: "tenant_empresa_peru_sac"
}

impl PgUserRepository {
    pub fn new(pool: PgPool, schema: String) -> Self {
        Self { pool, schema }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, _tenant_id: Uuid, user_id: Uuid) -> DomainResult<Option<User>> {
        let query = format!(
            "SELECT id, email, display_name, password_hash, status, last_login_at, created_at, updated_at
             FROM {}.users WHERE id = $1",
            self.schema
        );
        let row = sqlx::query_as::<_, UserRow>(&query)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(row.map(|r| r.into_user(_tenant_id)))
    }

    async fn find_by_email(&self, tenant_id: Uuid, email: &Email) -> DomainResult<Option<User>> {
        let query = format!(
            "SELECT id, email, display_name, password_hash, status, last_login_at, created_at, updated_at
             FROM {}.users WHERE email = $1",
            self.schema
        );
        let row = sqlx::query_as::<_, UserRow>(&query)
            .bind(email.value())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(row.map(|r| r.into_user(tenant_id)))
    }

    async fn save(&self, user: &User) -> DomainResult<()> {
        let status = match user.status {
            UserStatus::Active   => "active",
            UserStatus::Inactive => "inactive",
            UserStatus::Blocked  => "blocked",
        };
        let query = format!(
            r#"INSERT INTO {schema}.users (id, email, display_name, password_hash, status, last_login_at, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               ON CONFLICT (id) DO UPDATE SET
                   display_name  = EXCLUDED.display_name,
                   password_hash = EXCLUDED.password_hash,
                   status        = EXCLUDED.status,
                   last_login_at = EXCLUDED.last_login_at,
                   updated_at    = EXCLUDED.updated_at"#,
            schema = self.schema
        );
        sqlx::query(&query)
            .bind(user.id)
            .bind(user.email.value())
            .bind(&user.display_name)
            .bind(&user.password_hash)
            .bind(status)
            .bind(user.last_login_at)
            .bind(user.created_at)
            .bind(user.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, _tenant_id: Uuid, user_id: Uuid) -> DomainResult<()> {
        let query = format!("DELETE FROM {}.users WHERE id = $1", self.schema);
        sqlx::query(&query)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    display_name: String,
    password_hash: String,
    status: String,
    last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserRow {
    fn into_user(self, tenant_id: Uuid) -> User {
        User {
            id: self.id,
            tenant_id,
            email: Email::new(&self.email).expect("Invalid email in DB"),
            display_name: self.display_name,
            password_hash: self.password_hash,
            role_ids: vec![],
            status: match self.status.as_str() {
                "inactive" => UserStatus::Inactive,
                "blocked"  => UserStatus::Blocked,
                _          => UserStatus::Active,
            },
            last_login_at: self.last_login_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
