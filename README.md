# Kudamerah

A starter template for small, fast web apps: a Rust/Axum modular monolith on SQLite, shipped as one binary. A wizard generates a clean, ready-to-run project with the frontend you choose.

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

- **Server** (`apps/server`): Axum 0.8, SQLite via sqlx (WAL mode, migrations embedded in the binary), JSON errors, env-var config, tracing, graceful shutdown, and integration tests against in-memory SQLite.
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

## Deploy a generated project

Build the project on the server, or on a machine with the same operating system and CPU architecture. SolidJS projects need the frontend built first; the other frontend choices have no separate build step.

```sh
# SolidJS only
(cd apps/web && npm ci && npm run build)

# All frontend choices
cargo build --release --locked
```

Copy the release binary from `target/release/<project-name>-server` to the server. Also copy the matching web assets:

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
WEB_DIR=/opt/my-app/web
RUST_LOG=info
```

`WEB_DIR` can be omitted for an API-only project. Migrations are embedded in the binary and run automatically at startup.

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
Environment=WEB_DIR=/opt/my-app/web
Environment=RUST_LOG=info
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

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
