# Интерактивная доска (курсовой проект)

Монорепозиторий: Rust (Axum, луковая архитектура, PostgreSQL, JWT, WebSocket) + React/TypeScript (Vite).

## Структура

| Путь | Назначение |
|------|------------|
| [backend/](backend/) | Cargo workspace, крейт [server/](backend/server/) |
| [backend/server/src/domain/](backend/server/src/domain/) | Сущности, ошибки, трейты репозиториев |
| [backend/server/src/application/](backend/server/src/application/) | Сценарии (auth, доски) |
| [backend/server/src/infrastructure/](backend/server/src/infrastructure/) | SQLx, JWT, Argon2 |
| [backend/server/src/presentation/](backend/server/src/presentation/) | HTTP + WebSocket |
| [frontend/](frontend/) | SPA: логин, список досок, canvas, WS |
| [docker-compose.yml](docker-compose.yml) | `db` + `board-api` + `frontend` (статика) + `nginx` (edge-прокси) |
| [proxy/](proxy/) | nginx: `/api` → API, `/` → frontend |

## Быстрый старт (локально)

1. Поднять Postgres (или `docker compose up -d db` из корня `block/`).
2. Скопировать переменные: `cp .env.example .env` и выставить `DATABASE_URL`, `JWT_SECRET` (длинная строка).
3. Бэкенд:

```bash
cd backend/server
export DATABASE_URL=postgres://board:board@localhost:5432/board
export JWT_SECRET=dev-secret-change-me-in-production-min-16-chars
cargo run
```

4. Фронтенд (прокси `/api` на `127.0.0.1:3000` уже в `vite.config.ts`):

```bash
cd frontend
npm install
npm run dev
```

5. Тестовые данные (после миграций):

```bash
cd backend/server
export DATABASE_URL=...
cargo run --bin seed
```

Учётные записи сида: `alice@example.com` / `password123`, `bob@example.com` / `password123` (Bob — viewer на демо-доске).

## Docker / Podman Compose

Один `docker-compose.yml`, три приложенческих сервиса:

| Сервис | Роль |
|--------|------|
| `frontend` | Сборка SPA + **свой** nginx — только раздача файлов |
| `board-api` | Rust API |
| `nginx` | **Edge-прокси** (`./proxy`): `/api` → API, остальное → frontend |

```bash
podman compose up --build -d
podman compose build frontend && podman compose up -d frontend nginx   # UI
podman compose build nginx && podman compose up -d nginx                 # только прокси
podman compose build board-api && podman compose up -d board-api         # API
```

- Снаружи: `http://localhost:5173` (сервис `nginx`)
- API напрямую (отладка): `http://localhost:3000`

Сиды после старта БД:

```bash
cd backend/server
DATABASE_URL=postgres://board:board@localhost:5432/board cargo run --bin seed
```

## Тесты

Юнит-тесты домена: `cd backend/server && cargo test`

Интеграционные проверки ролей (нужен живой Postgres):

```bash
export DATABASE_URL=postgres://board:board@localhost:5432/board
export JWT_SECRET=integration-test-secret-32chars!!
cd backend/server
cargo test --test role_access -- --ignored
```

## Фаззинг (libFuzzer)

Цель `ws_message` парсит произвольные UTF-8 строки как JSON WebSocket-команд.

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
cd backend/server
cargo +nightly fuzz run ws_message -- -runs=1000
```

## Twelve-factor (кратко)

| Фактор | Как реализовано |
|--------|-----------------|
| I. Кодовая база | Один репозиторий `block/` |
| II. Зависимости | `Cargo.toml` / `package-lock.json`, явные версии |
| III. Конфиг | `DATABASE_URL`, `JWT_SECRET`, `HOST`, `PORT`, `FRONTEND_ORIGIN` через окружение ([config.rs](backend/server/src/config.rs)) |
| IV. Сторонние службы | PostgreSQL как attachable ресурс |
| V. Сборка/выпуск | Разделение `cargo build --release` и Docker-образа |
| VI. Процессы | Stateless API, комнаты WS в памяти процесса (демо) |
| VII. Привязка портов | `HOST`/`PORT` |
| VIII. Масштабирование | Горизонтально ограничено демо-WS (для прод — sticky sessions / Redis pubsub) |
| IX. Устойчивость | Graceful shutdown через `axum::serve` |
| X. Паритет dev/prod | Одинаковые контейнеры и переменные |
| XI. Журналы | `tracing` → stdout |
| XII. Админ-процессы | `cargo run --bin seed` |

## Деплой (Railway)

Продакшен: **PostgreSQL** (плагин) + три сервиса из репозитория — `board-api`, `frontend` (внутренний), `proxy` (публичный edge). Пошагово: [../docs/RAILWAY.md](../docs/RAILWAY.md). Шаблон переменных: [.env.production.example](.env.production.example).

## CI/CD (GitHub Actions)

Workflow [`.github/workflows/ci-cd.yml`](.github/workflows/ci-cd.yml): тесты на PR; на `main` — `railway up` для `board-api` и `frontend`. Сервис **proxy** деплоится автоматически через **Railway → GitHub** (root directory `proxy/`). Секрет: `RAILWAY_TOKEN` (project token).
