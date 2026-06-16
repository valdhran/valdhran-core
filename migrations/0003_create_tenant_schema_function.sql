-- Función para provisionar el schema de un nuevo tenant
-- DEC-004: schema-per-tenant. Esta función crea todas las tablas del tenant.
-- Se llama desde el caso de uso create_tenant (desde la API en Axum).

CREATE OR REPLACE FUNCTION public.provision_tenant_schema(p_schema_name TEXT)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    -- Crear schema
    EXECUTE format('CREATE SCHEMA IF NOT EXISTS %I', p_schema_name);

    -- Tabla de roles del tenant
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.roles (
            id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
            name            TEXT        NOT NULL,
            description     TEXT        NOT NULL DEFAULT '''',
            parent_role_id  UUID        REFERENCES %I.roles(id) ON DELETE SET NULL,
            is_system_role  BOOLEAN     NOT NULL DEFAULT FALSE,
            created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE (name)
        )', p_schema_name, p_schema_name);

    -- Tabla de permisos del tenant
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.permissions (
            id          UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
            resource    TEXT        NOT NULL,
            action      TEXT        NOT NULL,
            description TEXT        NOT NULL DEFAULT '''',
            created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE (resource, action)
        )', p_schema_name);

    -- Tabla pivot: roles ↔ permisos
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.role_permissions (
            role_id       UUID NOT NULL REFERENCES %I.roles(id)       ON DELETE CASCADE,
            permission_id UUID NOT NULL REFERENCES %I.permissions(id) ON DELETE CASCADE,
            PRIMARY KEY (role_id, permission_id)
        )', p_schema_name, p_schema_name, p_schema_name);

    -- Tabla de usuarios del tenant
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.users (
            id            UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
            email         TEXT        NOT NULL UNIQUE,
            display_name  TEXT        NOT NULL,
            password_hash TEXT        NOT NULL,
            status        TEXT        NOT NULL DEFAULT ''active''
                                      CHECK (status IN (''active'', ''inactive'', ''blocked'')),
            last_login_at TIMESTAMPTZ,
            created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )', p_schema_name);

    -- Tabla pivot: usuarios ↔ roles (corresponde a role_ids: Vec<Uuid> en User)
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.user_roles (
            user_id UUID NOT NULL REFERENCES %I.users(id) ON DELETE CASCADE,
            role_id UUID NOT NULL REFERENCES %I.roles(id) ON DELETE CASCADE,
            PRIMARY KEY (user_id, role_id)
        )', p_schema_name, p_schema_name, p_schema_name);

    -- Índices
    EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%s_users_email  ON %I.users (email)',  p_schema_name, p_schema_name);
    EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%s_users_status ON %I.users (status)', p_schema_name, p_schema_name);

    -- Insertar roles de sistema por defecto
    EXECUTE format('
        INSERT INTO %I.roles (name, description, is_system_role)
        VALUES
            (''super_admin'', ''Acceso total al tenant'', TRUE),
            (''admin'',       ''Administrador del tenant'', TRUE),
            (''user'',        ''Usuario estándar'', TRUE)
        ON CONFLICT (name) DO NOTHING
    ', p_schema_name);

END;
$$;

COMMENT ON FUNCTION public.provision_tenant_schema(TEXT) IS
    'Provisiona el schema completo de un nuevo tenant: roles, permisos, usuarios, pivots.
     Llamar desde create_tenant use case. DEC-004.';
