# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A personal expense-tracker HTTP API, built as a staged Rust learning project (see README.md
for the full roadmap and API surface). This is a learning exercise, not production code —
prefer explaining *why* Rust idioms work the way they do over silently writing code for the
user. When the user is implementing a new concept themselves, explain and show snippets for
them to type in rather than writing/editing the implementation files directly; direct edits
are fine for mechanical setup (Cargo.toml deps, `.gitignore`, config, docs).

M1 (in-memory API), M2 (SQLite persistence + categories as their own resource), and a full
integration test suite are done. The project deliberately expanded scope mid-M2: categories
moved from a fixed Rust enum to their own database-backed resource with CRUD endpoints, once a
closed category list felt too limiting.

M3 stretch goals: reports/totals, basic auth, a CLI client, Docker packaging. Reports/totals
(the first of these) is done — see the `report` bullets in Architecture/Database/Testing
below. The rest aren't started; no decision yet on which is next.

## Architecture

- **`models/{expense,category}.rs`** — plain data: `Expense`/`NewExpense`, `Category`/`NewCategory`.
  The `New*` types omit `id` (server-generated via `Uuid::new_v4()` in the handler) and only
  derive `Deserialize`, since they're request-body-only.
- **`models/report.rs`** — the reverse split from `New*`: `TotalReport`, `CategoryTotal`,
  `MonthTotal`, `CategoryMonthTotal` only derive `Serialize` (no `Deserialize`/`Clone`), since
  they're response-body-only — nothing ever deserializes a report from a request. `month` is a
  plain `String` in `"YYYY-MM"` form, not a `NaiveDate`, since a year-month bucket isn't a real
  calendar date. `ReportFilter` (`Deserialize`-only, `from`/`to`: `Option<NaiveDate>`) is the
  shared query-param struct for all four report endpoints, extracted via Axum's
  `Query<ReportFilter>` the same way `Json<NewExpense>` extracts a body.
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
- **`handlers/{expense,category,report}.rs`** — thin: extract `State`/`Path`/`Query`/`Json`,
  delegate to `AppState` methods, map `Option`/errors via `.ok_or(AppError::NotFound(...))?`.
  The four report handlers have no `NotFound`/`Conflict` path at all — an empty or nonsensical
  date range just yields an empty list / zero total, not an error, so `Result<_, AppError>` is
  only there for the `?` on a possible `sqlx::Error`. Each module also exposes its own
  `pub fn routes() -> Router<AppState>` bundling that resource's paths, written relative to
  where the module gets mounted (e.g. `report::routes()` uses `"/total"`, not `"/reports/total"`).
- **`lib.rs`** — the crate is split bin+lib specifically so integration tests (which compile
  as separate crates and can only see the public library surface) can reach the app. Exposes
  `pub mod {error,handlers,models,state}`, plus `app(state: AppState) -> Router` and
  `connect(database_url: &str) -> SqlitePool` (builds the pool with `PRAGMA foreign_keys = ON`
  and runs embedded migrations via `sqlx::migrate!()`). `app()` no longer lists routes
  directly — it composes each resource module's `routes()` via
  `.nest("/categories", category::routes())` / `.nest("/expenses", expense::routes())` /
  `.nest("/reports", report::routes())`, so each module owns its own path list and `lib.rs`
  just assembles them.
- **`main.rs`** — thin: loads `.env` via `dotenvy`, calls `connect()`, builds `AppState`,
  binds a `TcpListener`, calls `axum::serve(listener, app(state))`. Nothing here is needed by
  tests, which call `connect`/`app` directly instead of going through `main`.
- `state/expense.rs` has both `insert_expense` (plain `INSERT`, used by `create_expense`) and
  `update_expense` (a real `UPDATE ... SET ... WHERE id = ?`, used by `update_expense`'s
  handler) — these are deliberately separate methods, not one reused for both. An earlier
  version tried reusing `insert_expense` for updates too; it 500'd because `expenses.id` is a
  SQL `PRIMARY KEY` and a plain `INSERT` on an existing id violates that constraint (the old
  in-memory `HashMap`'s `.insert()` had silently-overwrite semantics that don't carry over to
  real SQL). If you're tempted to reuse a single method for insert-or-update again, don't —
  either add a real `UPDATE`, or use `INSERT OR REPLACE` explicitly and deliberately.

## Database

- Schema lives in `migrations/` (sqlx migrations, applied via `sqlx migrate run` or
  automatically on binary startup). `categories` table before `expenses` — the latter has
  `category_id REFERENCES categories (id)`.
- UUIDs are stored as SQLite `TEXT`; query macros need an explicit cast for non-obvious
  column types, e.g. `id as "id: Uuid"`, `date as "date: NaiveDate"` — the macro can't infer
  these from a plain `TEXT`/`INTEGER` column affinity.
- `DATABASE_URL` (in `.env`, gitignored) must be set for both `sqlx-cli` and the `query!`/
  `query_as!` macros to type-check against the real schema at compile time.
- Aggregate/computed columns (`SUM(...)`, `COALESCE(...)`) aren't real table columns, so the
  `query!`/`query_as!` macros can't infer their nullability from schema metadata the way they
  can for e.g. `categories.name` — they need an explicit `!` override (e.g.
  `COALESCE(SUM(amount), 0.0) as "total!: f64"`) to say "trust me, not null."
- `state/report.rs` deliberately splits by aggregation strategy rather than doing everything
  one way: `total_report`/`totals_by_category` push aggregation into SQL (`GROUP BY`/`SUM`),
  consistent with the SQL-first direction from M2. `totals_by_month`/`totals_by_category_month`
  only use SQL for the range `WHERE` filter (and the `categories` join, for the combined view);
  the actual month-bucketing and summing happens in Rust via a `HashMap` (keyed by the
  `"YYYY-MM"` string, or a `(category_id, month)` tuple for the combined view). SQLite's
  `strftime('%Y-%m', date)` would also work — it operates on the same ISO-8601 text `sqlx`
  writes for `NaiveDate` columns, no actual storage-format mismatch — but it'd need its own `!`
  override as a computed column, and Rust-side grouping was preferred instead.

## Testing

- `tests/api.rs` — integration tests driving the real `Router` in-process via
  `tower::ServiceExt::oneshot` (no port bound, no real TCP), asserting on real HTTP
  status/JSON bodies. A shared `request(&app, method, uri, body)` helper builds the request
  and returns `(StatusCode, serde_json::Value)`, to cut down repetition across tests.
- Each test builds its own fresh `sqlx::sqlite::SqlitePoolOptions` pool against
  `"sqlite::memory:"`, with **`.max_connections(1)`** — important: SQLite's `:memory:` gives
  each *connection* its own separate empty database, so a pool with more than one connection
  would silently split state across connections. Capping at 1 connection keeps every query in
  a test hitting the same in-memory DB. Migrations are run fresh per test via
  `sqlx::migrate!().run(&pool)`.
- Coverage so far: create/list for both resources, 404 (unknown expense/category id), 409
  (bogus `category_id` on create; deleting a category still referenced by an expense), update,
  delete. `cargo test` runs unit tests in `src/lib.rs`/`src/main.rs` (currently none written),
  each file under `tests/` as its own integration-test binary, and doc-tests.
- **`tests/reports.rs`** — same in-process pattern as `tests/api.rs`, in its own file/binary
  since integration tests each compile as a separate crate and can't share code without a
  `tests/common/` module (not introduced — `test_state()`/`request()` are duplicated here the
  same way, rather than adding shared test infrastructure for two files). Coverage: `total` on
  an empty DB (`0.0`, not an error — exercises the `COALESCE` override), summed and
  date-range-filtered; `by-category` grouping (and that a category with zero matching expenses
  doesn't appear, since it comes from `GROUP BY`, not a `LEFT JOIN`); `by-month` grouping and
  chronological sort; the combined `by-category-month` grouping; and `from > to` returning an
  empty/zero result rather than an error.

## Commands

- `cargo build` / `cargo run` — compile / run the server (port 3000)
- `cargo test` — run all tests; `cargo test <name>` for a single test
- `cargo clippy` — lint
- `sqlx migrate add <name>` — scaffold a new migration
- `sqlx migrate run` — apply pending migrations manually (also happens automatically on `cargo run`)
