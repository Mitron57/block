#!/usr/bin/env bash
# Выполняется на ВМ (из GitHub Actions по SSH).
set -euo pipefail

DEPLOY_PATH="${VM_DEPLOY_PATH:-$HOME/block}"
cd "$DEPLOY_PATH"

if [[ -z "${CR_IMAGE_API:-}" || -z "${CR_IMAGE_FRONTEND:-}" || -z "${CR_IMAGE_PROXY:-}" ]]; then
  echo "CR_IMAGE_* must be set" >&2
  exit 1
fi

if [[ -z "${DATABASE_URL:-}" || -z "${JWT_SECRET:-}" || -z "${FRONTEND_ORIGIN:-}" ]]; then
  echo "DATABASE_URL, JWT_SECRET, FRONTEND_ORIGIN must be set" >&2
  exit 1
fi

export DATABASE_URL JWT_SECRET FRONTEND_ORIGIN

docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d --remove-orphans

docker compose -f docker-compose.prod.yml ps
curl -fsS "http://127.0.0.1/health" && echo " — health ok"
