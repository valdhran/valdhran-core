-- Tabla global de refresh tokens
-- DEC-006: solo se almacena el hash SHA-256, nunca el token en claro
-- Tabla en schema public (no por tenant) para facilitar rotación y revocación global

CREATE TABLE IF NOT EXISTS public.refresh_tokens (
    id          UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID        NOT NULL,
    tenant_id   UUID        NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
    token_hash  TEXT        NOT NULL UNIQUE,
    expires_at  TIMESTAMPTZ NOT NULL,
    revoked_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user_id    ON public.refresh_tokens (user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_tenant_id  ON public.refresh_tokens (tenant_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token_hash ON public.refresh_tokens (token_hash);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires_at ON public.refresh_tokens (expires_at);

COMMENT ON TABLE  public.refresh_tokens            IS 'Refresh tokens para auth. DEC-006: solo hash SHA-256, rotación en cada uso.';
COMMENT ON COLUMN public.refresh_tokens.token_hash IS 'SHA-256 del token JWT. Nunca el token en claro.';
COMMENT ON COLUMN public.refresh_tokens.revoked_at IS 'NULL = token válido. NOT NULL = revocado (rotación o logout).';
