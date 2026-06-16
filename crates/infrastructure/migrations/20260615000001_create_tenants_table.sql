-- Tabla global de tenants en schema public
-- Un registro por empresa cliente del ERP

CREATE TABLE IF NOT EXISTS public.tenants (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug          VARCHAR(63)  NOT NULL UNIQUE,
    name          VARCHAR(255) NOT NULL,
    schema_name   VARCHAR(63)  NOT NULL UNIQUE,
    status        VARCHAR(20)  NOT NULL DEFAULT 'active'
                  CHECK (status IN ('active', 'suspended', 'cancelled')),
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tenants_slug ON public.tenants(slug);
CREATE INDEX IF NOT EXISTS idx_tenants_status ON public.tenants(status);

COMMENT ON TABLE public.tenants IS 'Registro global de todos los tenants del ERP. Cada tenant tiene su propio schema PostgreSQL.';
