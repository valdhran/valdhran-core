-- Tabla global de tenants (schema public)
CREATE TABLE IF NOT EXISTS public.tenants (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug        VARCHAR(63)  NOT NULL UNIQUE,
    name        VARCHAR(255) NOT NULL,
    schema_name VARCHAR(63)  NOT NULL UNIQUE,
    status      VARCHAR(20)  NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'suspended', 'cancelled')),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_tenants_slug ON public.tenants (slug);
