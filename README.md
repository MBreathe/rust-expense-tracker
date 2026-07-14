# rust-expense-tracker

A personal expense-tracker HTTP API, built as a learning project for Rust.

The goal isn't a finished product — it's a staged path through core Rust
concepts (ownership/borrowing, error handling, async, the crate ecosystem)
using a project simple enough to reason about but real enough to actually use.

## Roadmap

**M1 — In-memory API** (done)
CRUD endpoints for expenses, backed by a `HashMap` behind an `Arc<Mutex<...>>`,
no database. Covered: structs/enums, `Result`/`Option`/`?`-based error handling,
async routing with [Axum](https://github.com/tokio-rs/axum), JSON via `serde`.

**M2 — Persistence** (done)
Swapped the in-memory store for real SQLite via [`sqlx`](https://github.com/launchbadge/sqlx),
with compile-time-checked queries and migrations. Along the way, categories grew
into their own resource (see below) rather than staying a fixed enum — a scope
change from the original plan, made deliberately once a fixed category list felt
too limiting. Covered: async DB queries, connection pooling, SQL migrations,
foreign-key referential integrity, and mapping database errors (not found,
constraint violations) into proper HTTP status codes.

**Testing milestone** (done)
A full integration test suite (`tests/`) driving the real router in-process,
covering both resources plus every error path (404/409). Caught a real bug along
the way: `update_expense` was reusing `insert_expense`, which violated the
`expenses.id` primary key on update.

**M3 — Stretch goals** (pick as interest allows)
Expense totals/reports (by category, by month) — **done**. Basic auth — **done**.
Remaining: a small CLI client, Docker packaging.

## Data model

- **Category**: `id` (UUID), `name`.
- **Expense**: `id` (UUID), `amount`, `category_id` (UUID, references a `Category`),
  `description`, `date`.
- **User**: `id` (UUID), `username` (unique), `password_hash` (never returned by
  the API — every response exposes only `id`/`username`).

An expense's `category_id` must reference an existing category — enforced at the
database level via a foreign key (SQLite's FK enforcement is turned on explicitly
at connection time, since it's off by default).

## API

All endpoints below require a bearer token (`Authorization: Bearer <token>`)
except `POST /auth/register` and `POST /auth/login`.

- `POST /expenses` — create an expense
- `GET /expenses` — list expenses
- `GET /expenses/{id}` — get one expense
- `PUT /expenses/{id}` — update an expense (full replace)
- `DELETE /expenses/{id}` — delete an expense
- `POST /categories` — create a category
- `GET /categories` — list categories
- `DELETE /categories/{id}` — delete a category (fails if any expense still references it)
- `GET /reports/total` — total of all expenses, optionally filtered by `?from=&to=` (inclusive `YYYY-MM-DD` dates)
- `GET /reports/totals_by_category` — totals grouped by category
- `GET /reports/totals_by_month` — totals grouped by calendar month
- `GET /reports/totals_by_category_month` — totals grouped by category and month
- `POST /auth/register` — create a user (public)
- `POST /auth/login` — exchange a username/password for a JWT bearer token, valid 24h (public)

## Setup

1. Copy `.env` with `DATABASE_URL=sqlite://expenses.db` and a `JWT_SECRET` of your choice
   (already present locally, gitignored). `.env.dummy` shows the expected shape with
   placeholder values — it's committed, so never put a real secret in it.
2. Install the `sqlx` CLI once, globally: `cargo install sqlx-cli --no-default-features --features rustls,sqlite`.
3. `sqlx database create && sqlx migrate run` — creates `expenses.db` and applies `migrations/`.
4. `cargo run` — the binary also re-applies migrations automatically on startup via `sqlx::migrate!()`.

## Status

M1, M2, the test suite, and two M3 stretch goals (reports/totals, basic auth) are
complete: a working SQLite-backed CRUD API for expenses and categories, reporting
endpoints, and JWT-based auth gating every route but registration/login — with
foreign-key integrity and proper error responses (401 / 404 / 409 / 500).
