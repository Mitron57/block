#!/usr/bin/env bash
# Однократный сид демо-данных в Railway (из корня block/).
set -euo pipefail
cd "$(dirname "$0")/../backend"
railway link -p "${RAILWAY_PROJECT_ID:?}" -e "${RAILWAY_ENVIRONMENT:-production}" -s board-api
railway run cargo run --release --bin seed
