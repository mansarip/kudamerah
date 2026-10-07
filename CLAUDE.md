# Kudamerah

A Rust/Axum modular monolith on SQLite. Production builds can embed the frontend and ship as a single executable.

## Architecture principles

Prefer simple, low-resource, high-performance solutions. Scale the architecture only when there is real evidence the simpler solution is insufficient.

- **Backend:** Rust/Axum. One binary, a modular monolith. Not microservices.
- **Database:** SQLite first. Move to PostgreSQL only when the requirements justify it.
- **Frontend:** HTML/CSS/native browser APIs first. Use vanilla JS or HTMX for simple interactivity. Use SolidJS only for richer frontend apps. Use Rust/WASM only for computationally expensive browser workloads.
- **Dependencies:** keep dependencies, processes, infrastructure pieces and network boundaries to a minimum.
- **Avoid:** premature Redis, queues, microservices, containers, orchestration, heavy frameworks and unnecessary abstractions.
- **Optimize for:** low RAM/CPU usage, small bundles, fast startup, simple deployment and long-term maintainability.

## Layout

```
apps/server/            Axum binary (package `kudamerah-server`)
  build.rs              tracks frontend changes for embedded release builds
  src/lib.rs            app(): /api router + frontend
  src/config.rs         Config from env vars, with dev defaults
  src/db.rs             SQLite pool (WAL) + embedded migrations
  src/error.rs          AppError -> JSON error responses
  src/modules/          feature modules, one per domain area, mounted under /api
  src/web.rs            frontend: serves apps/web as static files # starter:vanilla
  src/web.rs            frontend: maud pages + htmx fragments; apps/web served at /static # starter:htmx
  src/web.rs            frontend: serves the Vite build with an index.html fallback # starter:solid
  migrations/           sqlx migrations, embedded at compile time, run on startup
  tests/                integration tests (in-memory SQLite, tower oneshot)
apps/web/               plain HTML/CSS/ES modules, no build step # starter:vanilla
apps/web/               static assets: CSS + vendored htmx.min.js # starter:htmx
apps/web/               SolidJS + Vite + TypeScript app # starter:solid
starter/                project generator + frontend overlays (template only) # starter:template
```

Shared library crates go in `crates/<name>/`. When you create the first one, also add `"crates/*"` to `members` in the root `Cargo.toml`. Cargo fails on a glob that matches nothing, so don't add it before that. Only extract a crate when two apps actually need the same code.

## Commands

```sh
cargo run                       # server on http://127.0.0.1:3000 (run from repo root)
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt
./build.sh                       # production wizard: embedded or separate frontend
```

<!-- starter:solid:begin -->
```sh
cd apps/web
npm run dev                     # UI with hot reload on :5173, proxies /api to :3000
npm run check                   # type-check
npm run build                   # -> apps/web/dist, served by the server
```

<!-- starter:solid:end -->
## Conventions

- **New feature:** create `src/modules/<name>.rs` (or a `<name>/` folder once it grows). It exposes `pub fn router() -> Router<AppState>`. Merge it in `src/modules/mod.rs`. A module owns its handlers and SQL. There are no separate repository/service layers until they are actually needed.
- **The example:** `src/modules/notes.rs` is a complete example module (routes, queries, validation, tests). Copy it for new features. Delete it once it is no longer useful, along with its migration, `tests/notes.rs` and its UI. <!-- starter:example -->
- **Handlers** return `Result<_, AppError>`. Use `?` on sqlx errors: `RowNotFound` becomes 404 and anything else becomes a logged 500. Shared state (`db`, `config`) comes from `State<AppState>`.
- **Queries** use runtime-checked `sqlx::query`/`query_as` with `?` bind parameters. Never build SQL from user input with `format!`.
- **Migrations:** add `apps/server/migrations/<YYYYMMDDHHMMSS>_<name>.sql`, preferring `STRICT` tables. Never edit a migration after it has been applied anywhere.
- **Config** comes only from env vars (see `.env.example`). Add new fields to `Config`; don't add a config crate.
- **Tests:** add tests per feature in `tests/<feature>.rs` using the helpers in `tests/common`. Each test gets a fresh in-memory database.
- **Frontend:** plain ES modules and CSS in `apps/web`. Don't add a bundler or npm. If an app grows into something rich, move it to SolidJS. Release builds may embed these files through the `embed-web` feature. <!-- starter:vanilla -->
- **Frontend:** pages and fragments are maud functions in `src/web.rs` (split it into `src/web/` as it grows). Return the smallest fragment that changes. maud escapes interpolated values, so never pass user input to `PreEscaped`. htmx is vendored at `apps/web/htmx.min.js`; upgrade it by replacing that file. <!-- starter:htmx -->
- **Frontend:** `apps/web` is Vite + SolidJS + TypeScript. Use solid-js primitives before adding libraries, and keep the bundle small. The server serves `apps/web/dist` with an `index.html` fallback, so client-side routes work. `./build.sh` builds the Vite app before the Rust release. <!-- starter:solid -->
- **Frontend:** none. This is an API-only service. If a UI is needed, follow the frontend principles above. <!-- starter:none -->
<!-- starter:template:begin -->

## Maintaining the template

This repo is a starter template. `starter/` is the generator, a package with no dependencies, run from a clone as `cargo starter new <dir>`. That command is an alias in `.cargo/config.toml`. See the docs at the top of `starter/src/main.rs`.

- **Base:** the repo root is the `vanilla` frontend plus the notes example. Here, `cargo run` and `cargo test` exercise that base, and `cargo test -p starter` tests the generator.
- **Template-only paths:** `starter/`, `.cargo/` and the README header image `kudamerah.png` are deleted from generated projects (`TEMPLATE_PATHS`).
- **Optional lines** carry marker comments. A line ending in a `starter:<feature>` comment is kept only when the feature is on. A `starter:<feature>:begin` … `:end` pair, each marker on its own comment line, wraps a block that is kept only when the feature is on. The features are `web`, `example`, `vanilla`, `htmx`, `solid`, `none` and `template`. `template` is always off and marks template-only content like this section. Removing any marker line must still leave valid code.
- **Frontend overlays:** `starter/frontends/<name>/` mirror repo paths. Generation deletes the vanilla files (`FRONTEND_PATHS` in the generator) and then copies the overlay on top. Some overlays copy base files: htmx copies `apps/server/Cargo.toml` and solid copies `.env.example`. When you change one of those base files, update the copy too.
- **Example files** are listed in `EXAMPLE_PATHS`. Keep that list in sync when you add example-only files.
- **The generated README** comes from `starter/README.project.md`. The `kudamerah` / `Kudamerah` / `kudamerah_` / `KUDAMERAH` tokens are renamed in every file, so don't use them for anything that should survive generation.
- **Before committing a template change,** run `starter/test.sh`. It generates all 8 frontend × example variants and runs fmt, clippy, tests, a leftover-token check and the npm build.
<!-- starter:template:end -->
