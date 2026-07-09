# rust-expense-tracker

A personal expense-tracker HTTP API, built as a learning project for Rust.

The goal isn't a finished product — it's a staged path through core Rust
concepts (ownership/borrowing, error handling, async, the crate ecosystem)
using a project simple enough to reason about but real enough to actually use.

## Roadmap

**M1 — In-memory API** (current stage)
CRUD endpoints for expenses, backed by a `Vec`/`HashMap` behind a `Mutex`,
no database. Focus: structs/enums, `Result`-based error handling, basic
async routing with [Axum](https://github.com/tokio-rs/axum), JSON via `serde`.

**M2 — Persistence**
Swap the in-memory store for SQLite via [`sqlx`](https://github.com/launchbadge/sqlx).
Focus: async DB queries, compile-time-checked SQL, migrations.

**M3 — Stretch goals** (pick as interest allows)
Category totals/reports, basic auth, a test suite, a small CLI client,
Docker packaging.

## Data model (planned)

An expense record: amount, category, description, date.

## API (planned, M1)

- `POST /expenses` — create an expense
- `GET /expenses` — list expenses
- `GET /expenses/:id` — get one expense
- `PUT /expenses/:id` — update an expense
- `DELETE /expenses/:id` — delete an expense

## Status

No code yet — project scaffolding in progress.
