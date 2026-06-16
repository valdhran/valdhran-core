#!/bin/bash
set -e

BASE_URL="${API_URL:-http://127.0.0.1:3000}"

echo "🧪 Smoke test contra $BASE_URL"
echo ""

# 1. Provisionar tenant
echo "1. POST /tenants"
TENANT_RESP=$(curl -s -X POST "$BASE_URL/tenants" \
  -H "Content-Type: application/json" \
  -d '{"slug":"empresa-test","name":"Empresa Test S.A.C."}')
echo "$TENANT_RESP" | python3 -m json.tool || echo "$TENANT_RESP"
echo ""

# 2. Login (fallará si no hay usuario — solo verifica que el endpoint responde)
echo "2. POST /auth/login"
LOGIN_RESP=$(curl -s -X POST "$BASE_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"tenant_slug":"empresa-test","email":"admin@test.com","password":"test1234"}')
echo "$LOGIN_RESP" | python3 -m json.tool || echo "$LOGIN_RESP"
echo ""

# 3. Refresh con token inválido (debe retornar 401)
echo "3. POST /auth/refresh (token inválido — espera 401)"
curl -s -X POST "$BASE_URL/auth/refresh" \
  -H "Content-Type: application/json" \
  -d '{"refresh_token":"token-invalido"}' | python3 -m json.tool
echo ""

echo "✅ Smoke test completado"
