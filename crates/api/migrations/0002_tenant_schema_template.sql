-- Función para crear el schema y tablas de un nuevo tenant
-- Se llama desde el use case ProvisionTenant después de insertar en public.tenants
CREATE OR REPLACE FUNCTION provision_tenant_schema(p_schema_name TEXT)
RETURNS void AS $$
BEGIN
    EXECUTE format('CREATE SCHEMA IF NOT EXISTS %I', p_schema_name);

    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.users (
            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email           VARCHAR(255) NOT NULL UNIQUE,
            display_name    VARCHAR(255) NOT NULL,
            password_hash   TEXT         NOT NULL,
            status          VARCHAR(20)  NOT NULL DEFAULT ''active''
                                CHECK (status IN (''active'', ''inactive'', ''blocked'')),
            last_login_at   TIMESTAMPTZ,
            created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
            updated_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
        )', p_schema_name);

    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.roles (
            id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name            VARCHAR(100) NOT NULL UNIQUE,
            description     TEXT,
            parent_role_id  UUID REFERENCES %I.roles(id),
            is_system_role  BOOLEAN      NOT NULL DEFAULT false,
            created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
            updated_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
        )', p_schema_name, p_schema_name);

    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.user_roles (
            user_id UUID NOT NULL REFERENCES %I.users(id) ON DELETE CASCADE,
            role_id UUID NOT NULL REFERENCES %I.roles(id) ON DELETE CASCADE,
            PRIMARY KEY (user_id, role_id)
        )', p_schema_name, p_schema_name, p_schema_name);

    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.refresh_tokens (
            id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id     UUID        NOT NULL REFERENCES %I.users(id) ON DELETE CASCADE,
            token_hash  TEXT        NOT NULL UNIQUE,
            expires_at  TIMESTAMPTZ NOT NULL,
            revoked     BOOLEAN     NOT NULL DEFAULT false,
            created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
        )', p_schema_name, p_schema_name);
END;
$$ LANGUAGE plpgsql;
