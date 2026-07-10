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

**M3 — Stretch goals** (pick as interest allows)
Expense totals/reports (by category, by month), basic auth, a test suite, a
small CLI client, Docker packaging.

## Data model

- **Category**: `id` (UUID), `name`.
- **Expense**: `id` (UUID), `amount`, `category_id` (UUID, references a `Category`),
  `description`, `date`.

An expense's `category_id` must reference an existing category — enforced at the
database level via a foreign key (SQLite's FK enforcement is turned on explicitly
at connection time, since it's off by default).

## API

- `POST /expenses` — create an expense
- `GET /expenses` — list expenses
- `GET /expenses/{id}` — get one expense
- `PUT /expenses/{id}` — update an expense (full replace)
- `DELETE /expenses/{id}` — delete an expense
- `POST /categories` — create a category
- `GET /categories` — list categories
- `DELETE /categories/{id}` — delete a category (fails if any expense still references it)

## Setup

1. Copy `.env` with `DATABASE_URL=sqlite://expenses.db` (already present locally, gitignored).
2. Install the `sqlx` CLI once, globally: `cargo install sqlx-cli --no-default-features --features rustls,sqlite`.
3. `sqlx database create && sqlx migrate run` — creates `expenses.db` and applies `migrations/`.
4. `cargo run` — the binary also re-applies migrations automatically on startup via `sqlx::migrate!()`.

## Status

M1 and M2 complete: a working SQLite-backed CRUD API for expenses and categories,
with foreign-key integrity and proper error responses (404 / 409 / 500).
