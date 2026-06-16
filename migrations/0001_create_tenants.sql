-- Tabla global de tenants (schema público)
-- DEC-004: cada tenant tendrá su propio schema, esta tabla es el registro central

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS public.tenants (
    id          UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug        TEXT        NOT NULL UNIQUE,
    name        TEXT        NOT NULL,
    schema_name TEXT        NOT NULL UNIQUE,
    status      TEXT        NOT NULL DEFAULT 'active'
                            CHECK (status IN ('active', 'suspended', 'cancelled')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tenants_slug   ON public.tenants (slug);
CREATE INDEX IF NOT EXISTS idx_tenants_status ON public.tenants (status);

COMMENT ON TABLE  public.tenants            IS 'Registro central de tenants. DEC-004: cada tenant tiene su propio schema PostgreSQL.';
COMMENT ON COLUMN public.tenants.slug       IS 'Identificador URL-friendly único, ej: empresa-sac';
COMMENT ON COLUMN public.tenants.schema_name IS 'Nombre del schema PostgreSQL del tenant, ej: tenant_empresa_sac';
COMMENT ON COLUMN public.tenants.status     IS 'active | suspended | cancelled';
