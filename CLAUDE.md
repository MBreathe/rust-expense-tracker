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

M3 stretch goals: reports/totals, basic auth, a CLI client, Docker packaging. Reports/totals,
basic auth, and a CLI client (see the `report`/`user`/`auth` bullets in
Architecture/Database/Testing, and the dedicated CLI Client section, below) are all done. The
API requires a bearer token on every route except `POST /auth/register` and `POST /auth/login`.
Docker packaging is the only thing left, not started.

## Architecture

- **`models/{expense,category}.rs`** — plain data: `Expense`/`NewExpense`, `Category`/`NewCategory`.
  The `New*` types omit `id` (server-generated via `Uuid::new_v4()` in the handler) and only
  derive `Deserialize`, since they're request-body-only.
- **`models/report.rs`** — the reverse split from `New*`: `TotalReport`, `CategoryTotal`,
  `MonthTotal`, `CategoryMonthTotal` are server response bodies — nothing on the *server* ever
  deserializes one — but all four also derive `Deserialize` (not just `Serialize`) so the CLI
  (`src/bin/cli/`, same crate) can deserialize its own HTTP responses straight into these same
  types instead of redefining them. `month` is a plain `String` in `"YYYY-MM"` form, not a
  `NaiveDate`, since a year-month bucket isn't a real calendar date. `ReportFilter`
  (`Deserialize`-only, `from`/`to`: `Option<NaiveDate>`) is the shared query-param struct for
  all four report endpoints, extracted via Axum's `Query<ReportFilter>` the same way
  `Json<NewExpense>` extracts a body.
- **`models/user.rs`** — `User { id, username }` (no `password_hash` field, so it's structurally
  impossible to leak one in a response) and `Credentials { username, password }`
  (`Deserialize`-only, deliberately shared by both `/auth/register` and `/auth/login` since both
  receive exactly the same shape) and `TokenResponse { token }`. `User`/`TokenResponse` derive
  both `Serialize` and `Deserialize` — `Deserialize` was added specifically for the CLI to
  reuse these types when parsing its own HTTP responses (same reasoning as `models/report.rs`
  above); the server itself only ever serializes them.
- **`state.rs`** — defines the single `AppState` struct (`Clone`, wraps a `sqlx::SqlitePool`
  plus a `pub jwt_secret: String`). `pool` stays private — only reached through `AppState`'s
  own methods — while `jwt_secret` is `pub` since `auth.rs` needs to read it directly from
  outside the `state` module. Registered once via `.with_state(...)` in `main.rs`; every
  handler reaches it via Axum's `State<AppState>` extractor.
- **`state/{expense,category}.rs`** — each contributes its own `impl AppState { ... }` block
  (Rust allows splitting a type's impl across files/modules) with that resource's async query
  methods (`list_*`, `insert_*`, `get_expense`, `remove_*`). Method names are resource-prefixed
  since both impls extend the same `AppState` type. Queries use `sqlx::query!`/`query_as!`
  (compile-time checked against the live `DATABASE_URL` schema); `remove_*` uses
  `DELETE ... RETURNING` to check-and-delete atomically in one query rather than a separate
  read then delete.
- **`state/user.rs`** — `insert_user` hashes the password with `argon2` before inserting
  (`SaltString::generate` for a fresh per-user salt, `Argon2::default().hash_password(...)`,
  stored as the standard PHC string via `.to_string()`). `verify_credentials` looks the user up
  by username and verifies the hash, returning `Option<Uuid>` rather than a `bool` — collapsing
  "no such username" and "wrong password" into the same `None`, which is what lets the login
  handler return one identical error message for both (see `auth.rs` and `handlers/user.rs`).
- **`error.rs`** — `AppError` (`NotFound(&'static str)`, `Conflict(&'static str)`,
  `Internal(sqlx::Error)`, `Unauthorized(&'static str)`) implements Axum's `IntoResponse`.
  `From<sqlx::Error> for AppError` inspects `sqlx::Error::as_database_error().kind()` to map
  SQLite foreign-key violations and `UNIQUE` violations (e.g. a duplicate username) both to
  `Conflict` (409); everything else becomes `Internal` (500).
- **`auth.rs`** — cross-cutting, like `error.rs`: not a resource itself, but every resource's
  handlers depend on it. Holds `Claims { sub: Uuid, exp: usize }` (the JWT payload), the
  `AuthUser(pub Uuid)` extractor — a hand-written `FromRequestParts<AppState>` impl, unlike
  `State`/`Path`/`Query` which are Axum's own built-in impls, since pulling the `Authorization`
  header, stripping the `Bearer ` prefix, and decoding/validating the JWT isn't something a
  built-in extractor does — and `issue_token(user_id, secret)` (wraps `jsonwebtoken::encode`,
  called from the login handler). `AuthUser`'s `Rejection` type is `AppError` directly, since
  it already implements `IntoResponse` — no need for a separate rejection type.
- **`handlers/{expense,category,report}.rs`** — thin: extract `State`/`Path`/`Query`/`Json`,
  delegate to `AppState` methods, map `Option`/errors via `.ok_or(AppError::NotFound(...))?`.
  The four report handlers have no `NotFound`/`Conflict` path at all — an empty or nonsensical
  date range just yields an empty list / zero total, not an error, so `Result<_, AppError>` is
  only there for the `?` on a possible `sqlx::Error`. Each module also exposes its own
  `pub fn routes() -> Router<AppState>` bundling that resource's paths, written relative to
  where the module gets mounted (e.g. `report::routes()` uses `"/total"`, not `"/reports/total"`).
  Every handler in these three files (12 total) also takes `AuthUser(_user): AuthUser` as a
  parameter — unused (nothing is scoped per-user yet), it's there purely so Axum runs the
  extractor, and its 401 on failure, before the handler body ever executes.
- **`handlers/user.rs`** — `register`/`login`, plus its own `routes()`. `register` calls
  `insert_user` and returns the created `User`; `login` calls `verify_credentials` and, on
  `Some(user_id)`, `auth::issue_token`; on `None`, returns
  `AppError::Unauthorized("invalid username or password")` — one message covering both "no
  such user" and "wrong password". Mounted via `.nest("/auth", user::routes())` in `lib.rs`, so
  the real paths are `/auth/register`/`/auth/login` (a deliberate deviation from an earlier,
  bare `/register`/`/login` design).
- **`lib.rs`** — the crate is split bin+lib specifically so integration tests (which compile
  as separate crates and can only see the public library surface) can reach the app. Exposes
  `pub mod {auth,error,handlers,models,state}`, plus `app(state: AppState) -> Router` and
  `connect(database_url: &str) -> SqlitePool` (builds the pool with `PRAGMA foreign_keys = ON`
  and runs embedded migrations via `sqlx::migrate!()`). `app()` composes each resource module's
  `routes()` via `.nest("/categories", ...)` / `.nest("/expenses", ...)` / `.nest("/reports",
  ...)` / `.nest("/auth", user::routes())`, so each module owns its own path list. `connect()`
  only builds the `SqlitePool` — `AppState::new(pool, jwt_secret)` happens separately (in
  `main.rs`, and in each test file's `test_state()`), since the JWT secret has nothing to do
  with the database connection.
- **`main.rs`** — thin: loads `.env` via `dotenvy`, reads `DATABASE_URL`/`JWT_SECRET`, calls
  `connect()`, builds `AppState::new(pool, jwt_secret)`, binds a `TcpListener`, calls
  `axum::serve(listener, app(state))`. Nothing here is needed by tests, which call
  `connect`/`app` directly instead of going through `main`.
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
  `category_id REFERENCES categories (id)`. `users` (`id`, `username` UNIQUE NOT NULL,
  `password_hash`) has no FK relationship to anything else — nothing is scoped per-user yet,
  the table exists purely to gate API access.
- UUIDs are stored as SQLite `TEXT`; query macros need an explicit cast for non-obvious
  column types, e.g. `id as "id: Uuid"`, `date as "date: NaiveDate"` — the macro can't infer
  these from a plain `TEXT`/`INTEGER` column affinity.
- `DATABASE_URL` (in `.env`, gitignored) must be set for both `sqlx-cli` and the `query!`/
  `query_as!` macros to type-check against the real schema at compile time. `JWT_SECRET` lives
  alongside it. `.env.dummy` is the committed template (git-tracked, unlike `.env`) — it needs
  a real-looking but fake value for each var, not the actual secret; a genuine `JWT_SECRET`
  briefly ended up there by mistake before being caught and replaced.
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
  status/JSON bodies. A shared `request(&app, method, uri, token, body)` helper builds the
  request (attaching an `Authorization: Bearer` header when `token` is `Some`) and returns
  `(StatusCode, serde_json::Value)`, to cut down repetition across tests. Since every route
  except `/auth/*` requires a token, each test also calls a `register_and_login(&app)` helper
  once at the top to get one.
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
  `tests/common/` module (not introduced — `test_state()`/`request()`, including the token
  handling, are duplicated here the same way, rather than adding shared test infrastructure
  for three files). Coverage: `total` on an empty DB (`0.0`, not an error — exercises the
  `COALESCE` override), summed and date-range-filtered; `by-category` grouping (and that a
  category with zero matching expenses doesn't appear, since it comes from `GROUP BY`, not a
  `LEFT JOIN`); `by-month` grouping and chronological sort; the combined `by-category-month`
  grouping; and `from > to` returning an empty/zero result rather than an error.
- **`tests/auth.rs`** — same in-process pattern, its own file/binary again. Covers register
  (success; duplicate username → 409), login (valid credentials → a token; wrong password and
  unknown username both → 401), and the `AuthUser` guard (missing/invalid/expired token all →
  401, valid token succeeds). One test compares raw response *bytes* rather than the usual
  JSON-parsed value — `AppError`'s plain-text error bodies aren't valid JSON, so
  `serde_json::from_slice(...).unwrap_or(Value::Null)` collapses both to `Value::Null`
  regardless of their actual text, which would make a "these two responses are identical"
  assertion pass trivially, catching nothing.

## CLI Client (`src/bin/cli/`)

A second binary in this same crate (not a separate workspace member), giving full CRUD parity
with the HTTP API via nested `clap` subcommands (`cli expense list`, `cli report total`, etc.).
Being in the same crate is what lets it reuse the server's own model types directly instead of
redefining them — see the `Deserialize` additions on `models/user.rs`/`models/report.rs` above.

- **`main.rs`** — `Cli::parse()`, then a nested `match` on `Command` (mirroring the nested
  `Subcommand` enums in `args.rs`) dispatching to the matching `commands::*` function. Prints
  and exits non-zero on a returned `CliError`.
- **`args.rs`** — the `clap` derive types. `Command::Expense(ExpenseCommand)` etc. — a tuple
  variant wrapping another `Subcommand` enum, marked `#[command(subcommand)]` — is the trick
  that makes two-level commands like `expense list` work. Fields with no `#[arg(...)]` attribute
  are positional (`id: Uuid` in `Get`/`Delete`); `#[arg(long)]` makes a named flag.
- **`http.rs`** — `request<T: DeserializeOwned>(method, path, auth, body) -> Result<T, CliError>`
  is the one function every command goes through: builds the URL from a base (defaults to
  `http://localhost:3000`, override via `API_BASE_URL`), attaches `Authorization: Bearer` when
  `auth` is true (reading the saved token, `CliError::NotLoggedIn` if there isn't one), sends
  the request, and deserializes the response body straight into whatever type the caller asked
  for. Also defines `CliError` (`NotLoggedIn`, `Request(reqwest::Error)`,
  `Api { status, body }`, `Io(std::io::Error)`) with a `Display` impl and `From` conversions,
  mirroring the server's own `AppError` pattern.
- **`token.rs`** — persists the bearer token as plain text at
  `~/.config/rust-expense-tracker/token` (via `std::env::var("HOME")`, not the `dirs` crate —
  this only ever needs to run on one machine). `login` writes it; every other command reads it.
- **`commands/{auth,category,expense,report}.rs`** — one function per endpoint, same
  resource-per-file split as the server's own `handlers/`. Each pulls response formatting into a
  private `format_*(&T) -> String` function (e.g. `format_expense`, `format_category_total`)
  rather than inlining `println!` — that's what makes the formatting logic unit-testable
  without capturing stdout, and it's also why those tests live inline in the same file
  (`#[cfg(test)] mod tests { use super::*; ... }`) rather than under `tests/`: unit tests need
  access to private items, which a separate file/crate (like everything under `tests/` already
  is) fundamentally can't reach. `report.rs`'s `build_query(from, to)` builds the `?from=&to=`
  query string — deliberately kept out of `http::request` itself, since it's specific to only
  these four callers.
- No persisted design doc for this feature (same as reports/auth) — the design was agreed
  conversationally and implemented directly.

## Commands

- `cargo build` / `cargo run` — compile / run the server (port 3000)
- `cargo build --bin cli` / `cargo run --bin cli -- <args>` — compile / run the CLI client.
  Note: `cargo check` does *not* rebuild the actual binary — if `cargo run --bin cli` ever
  silently does nothing, rebuild with `cargo build --bin cli` before assuming the code is wrong.
- `cargo test` — run all tests (server integration tests + CLI unit tests); `cargo test <name>`
  for a single test; `cargo test --bin cli` for just the CLI's unit tests.
- `cargo clippy --all-targets` — lint everything, including the CLI binary and its tests.
- `sqlx migrate add <name>` — scaffold a new migration
- `sqlx migrate run` — apply pending migrations manually (also happens automatically on `cargo run`)
