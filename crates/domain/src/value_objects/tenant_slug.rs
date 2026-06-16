use serde::{Deserialize, Serialize};
use crate::errors::{DomainError, DomainResult};

/// Slug único del tenant: solo letras minúsculas, números y guiones.
/// Ej: "empresa-peru-sac", "clinica-los-andes"
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantSlug(String);

impl TenantSlug {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into().trim().to_lowercase();
        if value.is_empty() || value.len() > 63 {
            return Err(DomainError::Validation("Slug must be 1-63 characters".into()));
        }
        if !value.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(DomainError::Validation(
                "Slug must contain only lowercase letters, numbers, and hyphens".into()
            ));
        }
        if value.starts_with('-') || value.ends_with('-') {
            return Err(DomainError::Validation("Slug cannot start or end with a hyphen".into()));
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TenantSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
