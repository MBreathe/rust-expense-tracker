# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A personal expense-tracker HTTP API, built as a staged Rust learning project (see README.md
for the full roadmap and API surface). This is a learning exercise, not production code —
prefer explaining *why* Rust idioms work the way they do over silently writing code for the
user. When the user is implementing a new concept themselves, explain and show snippets for
them to type in rather than writing/editing the implementation files directly; direct edits
are fine for mechanical setup (Cargo.toml deps, `.gitignore`, config, docs).

M1 (in-memory API) and M2 (SQLite persistence + categories as their own resource) are done.
The project deliberately expanded scope mid-M2: categories moved from a fixed Rust enum to
their own database-backed resource with CRUD endpoints, once a closed category list felt too
limiting. Don't jump ahead to M3 (auth, reports, tests, CLI client, Docker) unless asked.

## Architecture

- **`models/{expense,category}.rs`** — plain data: `Expense`/`NewExpense`, `Category`/`NewCategory`.
  The `New*` types omit `id` (server-generated via `Uuid::new_v4()` in the handler) and only
  derive `Deserialize`, since they're request-body-only.
- **`state.rs`** — defines the single `AppState` struct (`Clone`, wraps a `sqlx::SqlitePool`).
  Registered once via `.with_state(...)` in `main.rs`; every handler reaches it via Axum's
  `State<AppState>` extractor.
- **`state/{expense,category}.rs`** — each contributes its own `impl AppState { ... }` block
  (Rust allows splitting a type's impl across files/modules) with that resource's async query
  methods (`list_*`, `insert_*`, `get_expense`, `remove_*`). Method names are resource-prefixed
  since both impls extend the same `AppState` type. Queries use `sqlx::query!`/`query_as!`
  (compile-time checked against the live `DATABASE_URL` schema); `remove_*` uses
  `DELETE ... RETURNING` to check-and-delete atomically in one query rather than a separate
  read then delete.
- **`error.rs`** — `AppError` (`NotFound(&'static str)`, `Conflict(&'static str)`,
  `Internal(sqlx::Error)`) implements Axum's `IntoResponse`. `From<sqlx::Error> for AppError`
  inspects `sqlx::Error::as_database_error().kind()` to map SQLite foreign-key violations to
  `Conflict` (409); everything else becomes `Internal` (500).
- **`handlers/{expense,category}.rs`** — thin: extract `State`/`Path`/`Json`, delegate to
  `AppState` methods, map `Option`/errors via `.ok_or(AppError::NotFound(...))?`.
- **`main.rs`** — loads `.env` via `dotenvy`, builds the `SqlitePool` with
  `PRAGMA foreign_keys = ON` explicitly enabled (SQLite doesn't enforce FKs by default),
  runs embedded migrations via `sqlx::migrate!()` on startup, wires the `Router`.

## Database

- Schema lives in `migrations/` (sqlx migrations, applied via `sqlx migrate run` or
  automatically on binary startup). `categories` table before `expenses` — the latter has
  `category_id REFERENCES categories (id)`.
- UUIDs are stored as SQLite `TEXT`; query macros need an explicit cast for non-obvious
  column types, e.g. `id as "id: Uuid"`, `date as "date: NaiveDate"` — the macro can't infer
  these from a plain `TEXT`/`INTEGER` column affinity.
- `DATABASE_URL` (in `.env`, gitignored) must be set for both `sqlx-cli` and the `query!`/
  `query_as!` macros to type-check against the real schema at compile time.

## Commands

- `cargo build` / `cargo run` — compile / run the server (port 3000)
- `cargo test` — run all tests; `cargo test <name>` for a single test
- `cargo clippy` — lint
- `sqlx migrate add <name>` — scaffold a new migration
- `sqlx migrate run` — apply pending migrations manually (also happens automatically on `cargo run`)
