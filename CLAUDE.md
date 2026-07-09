# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

A personal expense-tracker HTTP API, built as a staged Rust learning project (see README.md
for the full roadmap). This is a learning exercise, not production code — prefer explaining
*why* Rust idioms work the way they do over silently writing idiomatic code for the user.

The project has been `cargo init`-ed but no real implementation exists yet: `src/main.rs` is
still the default `cargo init` stub (`println!("Hello, world!")`) and `Cargo.toml` has no
dependencies added.

## Planned architecture

Staged milestones, each introducing new concepts deliberately rather than all at once:

- **M1 (current target):** Axum web server, in-memory storage (`Vec`/`HashMap` behind a
  `Mutex`), `serde` for JSON. No database, no auth. This stage is deliberately kept simple
  so ownership/borrowing and `Result`-based error handling are the main things being learned.
  Planned endpoints: `POST /expenses`, `GET /expenses`, `GET /expenses/:id`,
  `PUT /expenses/:id`, `DELETE /expenses/:id`. An expense record has amount, category,
  description, date.
- **M2:** Replace in-memory storage with SQLite via `sqlx` (async, compile-time-checked
  queries) and migrations.
- **M3 (stretch, pick as interest allows):** category totals/reports, basic auth, tests,
  a small CLI client, Docker packaging.

Do not jump ahead to a later milestone's stack (e.g. adding a database or auth) unless asked —
the staging is intentional, not a TODO list to clear in one pass.

## Commands

- `cargo build` — compile
- `cargo run` — run the server
- `cargo test` — run all tests; `cargo test <name>` to run a single test
- `cargo clippy` — lint
