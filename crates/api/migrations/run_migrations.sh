#!/bin/bash
set -e

DB_URL="${DATABASE__URL:-postgres://valdhran:valdhran_dev@localhost:5432/valdhran}"

echo "Ejecutando migraciones en: $DB_URL"

psql "$DB_URL" -f "$(dirname "$0")/0001_create_tenants.sql"
psql "$DB_URL" -f "$(dirname "$0")/0002_tenant_schema_template.sql"

echo "✅ Migraciones completadas"
