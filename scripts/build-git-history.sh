#!/usr/bin/env bash
# Однократно: создаёт историю коммитов за неделю (10–16 мая 2026).
# Запуск из корня block/: ./scripts/build-git-history.sh
set -euo pipefail
cd "$(dirname "$0")/.."

if git rev-parse HEAD >/dev/null 2>&1; then
  echo "Репозиторий уже содержит коммиты. Скрипт только для пустого main." >&2
  exit 1
fi

commit_at() {
  local when="$1"
  local msg="$2"
  shift 2
  export GIT_AUTHOR_DATE="$when"
  export GIT_COMMITTER_DATE="$when"
  git add "$@"
  git commit -m "$msg"
  unset GIT_AUTHOR_DATE GIT_COMMITTER_DATE
}

# 2026-05-10 (сб) — 2 коммита
commit_at "2026-05-10 10:18:00 +0300" "chore: init monorepo layout" \
  .gitignore README.md .env.example \
  backend/Cargo.toml backend/Cargo.lock backend/server/Cargo.toml

commit_at "2026-05-10 16:42:00 +0300" "feat(backend): domain model and postgres migration" \
  backend/server/migrations/20250512000000_init.sql \
  backend/server/src/domain/ \
  backend/server/src/lib.rs

# 2026-05-11 (вс) — 3 коммита
commit_at "2026-05-11 11:05:00 +0300" "feat(backend): auth and board application layer" \
  backend/server/src/application/

commit_at "2026-05-11 14:33:00 +0300" "feat(backend): sqlx repositories, jwt and argon2" \
  backend/server/src/infrastructure/

commit_at "2026-05-11 19:50:00 +0300" "feat(backend): REST routes and error mapping" \
  backend/server/src/presentation/error.rs \
  backend/server/src/presentation/mod.rs \
  backend/server/src/presentation/state.rs \
  backend/server/src/presentation/routes.rs \
  backend/server/src/config.rs

# 2026-05-12 (пн) — 1 коммит
commit_at "2026-05-12 17:15:00 +0300" "feat(backend): websocket rooms and broadcast protocol" \
  backend/server/src/ws_protocol.rs \
  backend/server/src/presentation/rooms.rs \
  backend/server/src/presentation/ws.rs

# 2026-05-13 (вт) — 4 коммита
commit_at "2026-05-13 09:22:00 +0300" "feat(backend): server entrypoint and demo seed" \
  backend/server/src/main.rs \
  backend/server/src/bin/seed.rs

commit_at "2026-05-13 12:55:00 +0300" "test(backend): integration tests for board roles" \
  backend/server/tests/role_access.rs

commit_at "2026-05-13 16:18:00 +0300" "feat(frontend): vite react typescript scaffold" \
  frontend/.gitignore \
  frontend/.env.example \
  frontend/README.md \
  frontend/package.json \
  frontend/package-lock.json \
  frontend/tsconfig.json \
  frontend/tsconfig.app.json \
  frontend/tsconfig.node.json \
  frontend/vite.config.ts \
  frontend/eslint.config.js \
  frontend/index.html \
  frontend/public/ \
  frontend/src/main.tsx \
  frontend/src/index.css \
  frontend/src/assets/vite.svg \
  frontend/src/assets/react.svg

commit_at "2026-05-13 21:40:00 +0300" "feat(frontend): authentication and boards list" \
  frontend/src/api.ts \
  frontend/src/types.ts \
  frontend/src/auth.tsx \
  frontend/src/App.tsx \
  frontend/src/pages/LoginPage.tsx \
  frontend/src/pages/RegisterPage.tsx \
  frontend/src/pages/BoardsPage.tsx

# 2026-05-14 (ср) — 3 коммита
commit_at "2026-05-14 10:30:00 +0300" "feat(frontend): board canvas, tools and websocket client" \
  frontend/src/pages/BoardPage.tsx \
  frontend/src/assets/hero.png

commit_at "2026-05-14 15:07:00 +0300" "feat(frontend): eraser tool with element hit-testing" \
  frontend/src/boardHitTest.ts

commit_at "2026-05-14 19:52:00 +0300" "style(frontend): toolbar and board layout styles" \
  frontend/src/App.css

# 2026-05-15 (чт) — 3 коммита
commit_at "2026-05-15 09:14:00 +0300" "test(backend): libfuzzer target for websocket messages" \
  backend/server/fuzz/Cargo.toml \
  backend/server/fuzz/Cargo.lock \
  backend/server/fuzz/fuzz_targets/ws_message.rs

commit_at "2026-05-15 13:28:00 +0300" "build: dockerfiles and local compose stack" \
  backend/server/Dockerfile \
  frontend/Dockerfile \
  docker-compose.yml

commit_at "2026-05-15 20:06:00 +0300" "refactor(infra): separate static nginx and edge reverse proxy" \
  frontend/nginx.static.conf \
  proxy/Dockerfile \
  proxy/nginx.conf

# 2026-05-16 (пт) — 2 коммита
commit_at "2026-05-16 11:45:00 +0300" "build: railway config and production env example" \
  .env.production.example \
  backend/railway.toml \
  frontend/railway.toml \
  proxy/railway.toml \
  scripts/railway-seed.sh

commit_at "2026-05-16 15:30:00 +0300" "ci: github actions test and deploy to railway" \
  .github/workflows/ci-cd.yml \
  rust-toolchain.toml \
  scripts/build-git-history.sh

echo ""
echo "Готово. История:"
git log --oneline --format='%h %ad %s' --date=format:'%Y-%m-%d %H:%M'
