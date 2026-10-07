<p align="center">
  <img src="kudamerah.png" alt="Kudamerah">
</p>

A starter template for small, fast web apps: a Rust/Axum modular monolith on SQLite that can ship as one binary, including the frontend. A wizard generates a clean, ready-to-run project with the frontend you choose.

## Quick start: develop, build, deploy

Requires Git and Rust 1.94+ with Cargo. SolidJS projects also need Node 22+ and npm; the other frontends need no Node tooling. SQLite is bundled.

```sh
git clone https://github.com/mansarip/kudamerah ~/kudamerah
cd ~/kudamerah
cargo starter new ~/projects/my-app --frontend vanilla --yes
cd ~/projects/my-app
cargo run                       # open http://127.0.0.1:3000
```

The first run creates `data/my-app.db` and applies migrations automatically. After stopping the dev server with Ctrl+C:

```sh
cargo test
./build.sh --single              # target/release/my-app-server
```

Build on the target server or a machine with the same OS and CPU architecture, then copy `target/release/my-app-server` to the server. Run it with `DATABASE_PATH` pointing to a persistent, writable location:

```sh
DATABASE_PATH=/var/lib/my-app/my-app.db BIND_ADDR=127.0.0.1:3000 RUST_LOG=info ./my-app-server
```

For a persistent service and public HTTPS, follow [Deploy a generated project](#deploy-a-generated-project) below. The binary includes the frontend; the SQLite database stays on disk separately.

## Create a project

Clone the template once and keep it around. `git pull` gets template updates.

```sh
git clone https://github.com/mansarip/kudamerah ~/kudamerah
```

To create a project, run the wizard from inside the clone. Point it at where the new project should go:

```sh
cd ~/kudamerah
cargo starter new ~/projects/my-app
```

```
New project wizard (Enter accepts the [default])

Project name [my-app]:
Description (optional): Booking system for a small clinic
Frontend:
  1) vanilla  static HTML/CSS/JS, no build step
  2) htmx     server-rendered HTML (maud) + htmx
  3) solid    SolidJS + Vite SPA (needs Node)
  4) none     JSON API only
Choose [1]: 2
Include the notes example (CRUD, migration, tests)? [Y/n]:
Commit the result to git? [Y/n]:
```

To skip the questions, pass flags: `cargo starter new ~/projects/my-app --frontend htmx --no-example --yes`. `cargo starter --help` lists all flags.

`cargo starter` is an alias, defined in `.cargo/config.toml`, for `cargo run -p starter --`. Nothing is installed globally. The first run compiles the generator in a few seconds; it has no dependencies. The new project is a clean copy with its own git history, and it has no link back to this template.

## What you get

- **Server** (`apps/server`): Axum 0.8, SQLite via sqlx (WAL mode, migrations embedded in the binary), optional embedded frontend assets, JSON errors, env-var config, tracing, graceful shutdown, and integration tests against in-memory SQLite.
- **Frontend** (`apps/web`): one of

  | Choice    | Stack                                           | Build step |
  |-----------|-------------------------------------------------|------------|
  | `vanilla` | HTML, CSS, ES modules                           | none       |
  | `htmx`    | maud (compile-time HTML) + htmx 2, vendored     | none       |
  | `solid`   | SolidJS + Vite + TypeScript                     | npm        |
  | `none`    | JSON API only                                   | n/a        |

- **Example** (optional): a notes CRUD module with a migration, tests and UI, showing the module pattern.
- **CLAUDE.md:** architecture principles and conventions for AI assistants.
- **Footprint:** the release build is a ~4 MB binary using ~5 MB RAM.

Every name is renamed from `kudamerah` to your project name: crate, binary, database file and titles. The generator itself (`starter/`) is not included in the generated project.

## Coming from JavaScript / TypeScript

The backend is Rust, but the workflow should feel familiar. Cargo manages dependencies, compiles the server and runs tests. You do not need a separate dependency-install command before `cargo run`.

| In JS/TS | In this project |
|----------|-----------------|
| `package.json` / `package-lock.json` | `Cargo.toml` / `Cargo.lock` for Rust packages and pinned dependencies |
| npm workspaces | A Cargo workspace; `apps/server` is a Rust package (a crate) |
| `npm run dev` | `cargo run` builds and starts the backend; restart it after Rust changes |
| `tsc --noEmit` | `cargo check` checks Rust without producing the final executable |
| ESLint / Prettier | `cargo clippy --all-targets -- -D warnings` / `cargo fmt` |
| Express or Fastify routes | Axum routers and handlers in `apps/server/src/modules/` |
| TypeScript interfaces and JSON conversion | Rust structs with Serde's `Serialize` / `Deserialize` |
| `Promise<T>` and `async` / `await` | Rust futures and `async` / `.await`, run by Tokio |
| `process.env` | `std::env`, read in `apps/server/src/config.rs` |

Rust types are checked at compile time. `Option<T>` represents a value that may be absent; `Result<T, E>` represents success or an error. The `?` operator propagates errors to the caller, and this project's `AppError` converts handler errors into HTTP responses. Ownership and borrowing (`&T`) let code use values without a garbage collector; use compiler messages to see when a value should be borrowed, moved or cloned.

Start with `apps/server/src/modules/notes.rs` if you included the example: it contains the routes, request/response types and database queries for one feature. Register new feature modules in `apps/server/src/modules/mod.rs`. SQL migrations live in `apps/server/migrations` and run at startup; sqlx executes SQL directly rather than providing a JavaScript-style ORM.

For a familiar frontend workflow, choose `--frontend solid` when generating the project. Keep `cargo run` running in one terminal, then use a second terminal in the generated project:

```sh
cd apps/web
npm install
npm run dev                     # http://localhost:5173; /api proxies to Rust
npm run check                   # TypeScript check
```

Vite provides frontend hot reload. From the project root, `./build.sh --single` builds both the frontend and backend for deployment; Node and npm are needed at build time, but are not needed to run the server binary. With `vanilla`, edit `apps/web` and refresh the browser; with `htmx`, HTML is rendered by Rust using maud, so template changes require restarting `cargo run`.

Configuration comes from environment variables. `.env.example` documents them, but `.env` files are **not loaded automatically**: export variables in your shell or configure them in your service. For example, `BIND_ADDR=127.0.0.1:4000 cargo run` changes the backend port.

## Deploy a generated project

Every generated project includes a production build wizard. Run it on the server, or on a machine with the same operating system and CPU architecture:

```sh
./build.sh
```

It asks how the frontend should be packaged:

```text
Production build:
  1) single    embed the frontend in the server binary
  2) separate  keep the server binary and frontend files separate
Choose [1]:
```

For a non-interactive build, use `./build.sh --single` or `./build.sh --separate`. SolidJS projects automatically run `npm ci` and `npm run build` first. API-only projects skip the question because they already produce one executable.

Both choices create the server executable at `target/release/<project-name>-server`. A single build needs only that executable. A separate build also needs the matching web assets:

| Frontend | Files to deploy |
|----------|-----------------|
| `vanilla` | `apps/web/` |
| `htmx` | `apps/web/` |
| `solid` | `apps/web/dist/` |
| `none` | no web assets |

Use absolute paths in production. The SQLite database directory must be persistent and writable by the service user. For example:

```ini
BIND_ADDR=127.0.0.1:3000
DATABASE_PATH=/var/lib/my-app/my-app.db
RUST_LOG=info
```

For a separate build, also set `WEB_DIR=/opt/my-app/web`. A single build ignores `WEB_DIR` in production because its assets are compiled into the executable. Migrations are embedded in both builds and run automatically at startup.

A minimal `systemd` service looks like this (replace `my-app` with your project name):

```ini
[Unit]
Description=My App
After=network.target

[Service]
User=my-app
Group=my-app
ExecStart=/opt/my-app/my-app-server
Environment=BIND_ADDR=127.0.0.1:3000
Environment=DATABASE_PATH=/var/lib/my-app/my-app.db
Environment=RUST_LOG=info
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Add `Environment=WEB_DIR=/opt/my-app/web` under `[Service]` when using a separate build.

After saving it as `/etc/systemd/system/my-app.service`, start it with:

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now my-app
curl --fail http://127.0.0.1:3000/api/health
```

Put a reverse proxy such as Caddy or nginx in front of the service for HTTPS and compression. Run a single application instance against the local SQLite database, and keep the database on a persistent disk. Back it up with SQLite's online backup command:

```sh
sqlite3 /var/lib/my-app/my-app.db ".backup /path/to/backups/my-app.db"
```

## Working on the template

```sh
cargo run                 # the base: vanilla frontend + notes example
cargo test                # base tests
cargo test -p starter     # generator tests
starter/test.sh           # generate and check all 8 variants
```

The template mechanics (marker comments, frontend overlays) are described in CLAUDE.md under "Maintaining the template".
