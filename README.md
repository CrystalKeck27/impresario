<picture>
    <source srcset="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_Solid_White.svg" media="(prefers-color-scheme: dark)">
    <img src="https://raw.githubusercontent.com/leptos-rs/leptos/main/docs/logos/Leptos_logo_RGB.svg" alt="Leptos Logo">
</picture>

# Leptos Axum Starter Template

This is a template for use with the [Leptos](https://github.com/leptos-rs/leptos) web framework and the [cargo-leptos](https://github.com/akesson/cargo-leptos) tool using [Axum](https://github.com/tokio-rs/axum).

## Creating your template repo

If you don't have `cargo-leptos` installed you can install it with

```bash
cargo install cargo-leptos --locked
```

Then run
```bash
cargo leptos new --git https://github.com/leptos-rs/start-axum
```

to generate a new project template.

```bash
cd impresario
```

to go to your newly created project.
Feel free to explore the project structure, but the best place to start with your application code is in `src/app.rs`.
Additionally, Cargo.toml may need updating as new versions of the dependencies are released, especially if things are not working after a `cargo update`.

## Running your project

```bash
cargo leptos watch
```

The server uses Chromaprint's `fpcalc` executable to fingerprint selected audio files and `ACOUSTID_APPLICATION_KEY` to look up matches. Install Chromaprint on the server and set an AcoustID application key in the server environment before running the app. Keep the key out of client-side code and source control.

## Local PostgreSQL database

The repository includes a PostgreSQL service for development. The Leptos app runs on the host and uses PostgreSQL when the directory fingerprinting feature is started.

Enter the Nix development shell to get the Docker CLI and Compose v2 plugin, Podman and its Compose provider, and PostgreSQL client tools. Nix supplies command-line tools, but it does not install or start a container runtime or configure host permissions. Install and enable Docker Engine (including its Compose plugin) or Podman on the host first. On macOS and Windows, the runtime generally needs its own VM. The current flake development shell targets x86_64 Linux.

Copy `.env.example` to `.env` in the repository root. The example credentials are for local development only; never reuse them in production. Compose loads `.env` automatically. Start PostgreSQL with either runtime:

```sh
docker compose up -d db
# or
podman compose up -d db
```

The database is published only on `127.0.0.1:5432`. Check its health and connect from the host with `psql`:

```sh
docker compose ps
psql "$(grep '^DATABASE_URL=' .env | cut -d= -f2-)" -c 'select 1;'
```

The PostgreSQL schema is available both as `scripts/create-tables.sql` and as a Diesel migration. To create the tables through Diesel, install the Diesel CLI with PostgreSQL support and run:

```sh
DATABASE_URL="$(grep '^DATABASE_URL=' .env | cut -d= -f2-)" diesel migration run
```

The checked-in Diesel table definitions are in `src/schema.rs`. Set `DATABASE_URL` in the server environment; the app creates its connection pool when a fingerprint scan is requested. Apply migrations manually with Diesel CLI as shown above. The app does not run migrations automatically, and the database must already contain the fingerprint tables.

The `/fingerprint` page recursively scans regular files under the supplied directory, skips canonical paths that already have an AcoustID fingerprint row, and runs up to eight `fpcalc` processes concurrently. Each successful result is stored in `acoustid_fingerprints`; file paths are stored in `impresario_files`. Progress is polled while the server-side job runs. Install Chromaprint so `fpcalc` is available on the server `PATH`. Directory paths are server-side filesystem paths, so expose this feature only to trusted users or add an allowed-root policy before public deployment.

Use `podman compose` instead of `docker compose` for the equivalent Podman commands. Podman's `compose` command delegates to an external provider; this dev shell includes `podman-compose`. If provider selection is ambiguous, set `PODMAN_COMPOSE_PROVIDER=podman-compose` for the command.

Stop the service while preserving its database with `docker compose down` or `podman compose down`. The named volume keeps data across container restarts and recreation. To permanently delete the local database and all its data, run `docker compose down -v` or `podman compose down -v`.

The example `DATABASE_URL` uses `localhost`, which is correct for the host-run app. If the app is later run as a container in this Compose project, it should connect to the database using the service hostname `db` instead. Keep real production credentials in a secrets manager or deployment environment, not in `.env.example` or the Compose file.

## Installing Additional Tools

By default, `cargo-leptos` uses `nightly` Rust, `cargo-generate`, and `sass`. If you run into any trouble, you may need to install one or more of these tools.

1. `rustup toolchain install nightly --allow-downgrade` - make sure you have Rust nightly
2. `rustup target add wasm32-unknown-unknown` - add the ability to compile Rust to WebAssembly
3. `cargo install cargo-generate` - install `cargo-generate` binary (should be installed automatically in future)
4. `npm install -g sass` - install `dart-sass` (should be optional in future
5. Run `npm install` in end2end subdirectory before test

## Compiling for Release
```bash
cargo leptos build --release
```

Will generate your server binary in target/release and your site package in target/site

## Testing Your Project
```bash
cargo leptos end-to-end
```

```bash
cargo leptos end-to-end --release
```

Cargo-leptos uses Playwright as the end-to-end test tool.
Tests are located in end2end/tests directory.

## Executing a Server on a Remote Machine Without the Toolchain
After running a `cargo leptos build --release` the minimum files needed are:

1. The server binary located in `target/server/release`
2. The `site` directory and all files within located in `target/site`

Copy these files to your remote server. The directory structure should be:
```text
impresario
site/
```
Set the following environment variables (updating for your project as needed):
```sh
export LEPTOS_OUTPUT_NAME="impresario"
export LEPTOS_SITE_ROOT="site"
export LEPTOS_SITE_PKG_DIR="pkg"
export LEPTOS_SITE_ADDR="127.0.0.1:3000"
export LEPTOS_RELOAD_PORT="3001"
export ACOUSTID_APPLICATION_KEY="your-acoustid-application-key"
export DATABASE_URL="postgres://user:password@host:5432/database"
```
Apply the Diesel migration before starting fingerprint scans. Install Chromaprint so `fpcalc` is available on the server's `PATH`, then run the server binary.

## Licensing

This template itself is released under the Unlicense. You should replace the LICENSE for your own application with an appropriate license if you plan to release it publicly.
