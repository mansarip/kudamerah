# Kudamerah

{{description}}

A Rust/Axum modular monolith on SQLite that can ship as a single binary, including the frontend.
The frontend is plain HTML/CSS/JS in `apps/web`, served as-is with no build step. <!-- starter:vanilla -->
The frontend is server-rendered HTML ([maud](https://maud.lambda.xyz)) enhanced with [htmx](https://htmx.org). <!-- starter:htmx -->
The frontend is [SolidJS](https://www.solidjs.com) + Vite in `apps/web`, built to `apps/web/dist` and served by the server. <!-- starter:solid -->
This is a JSON API only, with no frontend. <!-- starter:none -->

## Develop

```sh
cargo run          # http://127.0.0.1:3000
cargo test
cargo clippy --all-targets -- -D warnings
```

<!-- starter:solid:begin -->
For the frontend with hot reload, run this in a second terminal:

```sh
cd apps/web
npm install
npm run dev        # http://localhost:5173, proxies /api to cargo run
npm run check      # type-check
```

<!-- starter:solid:end -->
Requires Rust 1.94+. <!-- starter:!solid -->
Requires Rust 1.94+ and Node 22+. <!-- starter:solid -->
SQLite is bundled, so it needs no system library. The database is created at `data/kudamerah.db` on first run. Migrations in `apps/server/migrations` are embedded in the binary and run on startup.

## Structure

| Path          | What                                                            |
|---------------|-----------------------------------------------------------------|
| `apps/server` | Axum server. JSON API under `/api`, feature modules in `src/modules` |
| `apps/web`    | Static HTML/CSS/JS, no build step | <!-- starter:vanilla -->
| `apps/web`    | Static assets (CSS, vendored htmx) | <!-- starter:htmx -->
| `apps/web`    | SolidJS + Vite + TypeScript app | <!-- starter:solid -->
| `crates/`     | Shared Rust crates, created when first needed                   |

## Configuration

Set through environment variables (see `.env.example`). The defaults are for local dev:

| Variable        | Default                 |
|-----------------|-------------------------|
| `BIND_ADDR`     | `127.0.0.1:3000`        |
| `DATABASE_PATH` | `data/kudamerah.db`     |
| `WEB_DIR`       | `apps/web`              | <!-- starter:vanilla -->
| `WEB_DIR`       | `apps/web`              | <!-- starter:htmx -->
| `WEB_DIR`       | `apps/web/dist`         | <!-- starter:solid -->
| `RUST_LOG`      | `info,tower_http=debug` |

`DATABASE_PATH=:memory:` gives a throwaway in-memory database.

## Deploy

Run the production build wizard:

```sh
./build.sh
```

For projects with a frontend, it asks whether to embed the frontend in one executable or keep the server and frontend files separate. Use `./build.sh --single` or `./build.sh --separate` in CI. SolidJS dependencies and assets are built automatically. <!-- starter:web -->
API-only projects build their single executable without asking a packaging question. <!-- starter:none -->

Both choices write the server to `target/release/kudamerah-server`. With `--single`, that executable is the only application artifact to deploy. With `--separate`, also copy `apps/web/` and set `WEB_DIR` on the server. <!-- starter:vanilla -->
Both choices write the server to `target/release/kudamerah-server`. With `--single`, that executable is the only application artifact to deploy. With `--separate`, also copy `apps/web/` and set `WEB_DIR` on the server. <!-- starter:htmx -->
Both choices write the server to `target/release/kudamerah-server`. With `--single`, that executable is the only application artifact to deploy. With `--separate`, also copy `apps/web/dist/` and set `WEB_DIR` on the server. <!-- starter:solid -->
Copy `target/release/kudamerah-server` to the server. <!-- starter:none -->

Set `DATABASE_PATH` to an absolute path and run the binary under systemd. The database directory must be persistent and writable by the service user. Put a reverse proxy (e.g. Caddy) in front for TLS and compression.

Back up the database with `sqlite3 data/kudamerah.db ".backup backup.db"`.
