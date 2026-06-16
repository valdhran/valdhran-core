# valdraegorn-core

Motor principal de Valdraegorn ERP — 100% Rust.

## Arquitectura

Arquitectura hexagonal con 3 crates:

- `valdraegorn-domain`: entidades puras, traits de repositorio, errores de dominio. Sin dependencias externas.
- `valdraegorn-application`: casos de uso y workflows. Depende solo del domain.
- `valdraegorn-infrastructure`: implementaciones concretas (PostgreSQL via SQLx, JWT, Argon2). Depende de domain + application.

## Stack
- Rust 2021 edition
- Tokio (async runtime)
- SQLx (PostgreSQL, compile-time queries)
- Axum (HTTP — en valdraegorn-api)
- Argon2 (password hashing)
- JWT (autenticación)
