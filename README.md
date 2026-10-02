# Kudamerah

A starter template for small, fast web apps: a Rust/Axum modular monolith on SQLite, shipped as one binary. A wizard generates a clean, ready-to-run project with the frontend you choose.

## Create a project

Install the generator once. It remembers where this template lives:

```sh
cargo install --path starter
```

Then run it from anywhere:

```sh
kudamerah new my-app
```

```
kudamerah: new project wizard (Enter accepts the [default])

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

To skip the questions, pass flags: `kudamerah new my-app --frontend htmx --no-example --yes`. `kudamerah --help` lists all flags.

Other ways to run it:

- **Without installing:** `cargo run -p kudamerah -- new ../my-app`
- **From a clone or a GitHub "Use this template" copy:** run `cargo run -p kudamerah -- init` inside it.
- **From a remote template:** `kudamerah new my-app --template https://github.com/mansarip/kudamerah`

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

## Working on the template

```sh
cargo run                 # the base: vanilla frontend + notes example
cargo test                # base tests
cargo test -p kudamerah   # generator tests
starter/test.sh           # generate and check all 8 variants
```

The template mechanics (marker comments, frontend overlays) are described in CLAUDE.md under "Maintaining the template".
