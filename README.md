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

The repository includes a PostgreSQL service for development. The Leptos app still runs on the host; this service only provides the database and is not yet connected to the app.

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

Use `podman compose` instead of `docker compose` for the equivalent Podman commands. Podman's `compose` command delegates to an external provider; this dev shell includes `podman-compose`. If provider selection is ambiguous, set `PODMAN_COMPOSE_PROVIDER=podman-compose` for the command.

Stop the service while preserving its database with `docker compose down` or `podman compose down`. The named volume keeps data across container restarts and recreation. To permanently delete the local database and all its data, run `docker compose down -v` or `podman compose down -v`.

The example `DATABASE_URL` uses `localhost`, which is correct for the host-run app. If the app is later run as a container in this Compose project, it should connect to the database using the service hostname `db` instead. Keep real production credentials in a secrets manager or deployment environment, not in `.env.example` or the Compose file.

## Identify music

Open `/identify`, enter a source directory and a destination directory on the server, then choose **Run identification**. The scan is recursive and currently considers files with `.mp3`, `.flac`, `.ogg`, `.opus`, `.m4a`, `.mp4`, `.aac`, `.wav`, `.aiff`, `.aif`, `.wma`, `.ape`, and `.wv` extensions (case-insensitive). It uses `fpcalc` and the configured AcoustID key to find MusicBrainz recording IDs.

Destination subdirectories mirror the source directory. For example, `jazz/track.flac` produces `jazz/track.flac.recording.mbid`, containing the selected recording MBID as UTF-8 text. A unique recording match is written automatically; multiple distinct recording IDs require choosing one in the page. No recording ID is shown as not found. Existing sidecars with the same ID are treated as already written, while differing contents require an explicit **Replace sidecar** confirmation. The destination cannot be inside the source directory.

Both paths refer to the machine running the Leptos server, not the browser user's local filesystem. Only run this filesystem feature in a trusted environment; the server must have access to the chosen paths.

## Group release tracks

Open `/group` after identification and choose the directory containing `.recording.mbid` sidecars plus a separate destination directory. The page searches MusicBrainz releases and their media for the input recordings, then writes unambiguous `<audio filename>.track.mbid` sidecars into a mirrored directory tree. Release options link to MusicBrainz. A release containing every distinct input recording is preferred, with the largest total track count ranked first; if none covers all inputs, a sole candidate for a recording is selected automatically and multiple candidates remain selectable per file. A release choice is propagated to matching files that have not been manually changed, and manual release or track selections remain in place. Newly resolved manual selections can be saved per file. Repeated recording occurrences must be assigned explicitly.

Grouping requires network access to the MusicBrainz web service. Requests use the application User-Agent and are serialized with at least 1.1 seconds between calls. MusicBrainz errors are shown rather than treated as empty results. Track sidecars with identical contents are idempotent; replacing a different track ID requires confirmation. As with `/identify`, paths are on the server and the destination cannot be inside the source tree. Use this feature only in a trusted environment.

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
```
Install Chromaprint so `fpcalc` is available on the server's `PATH`, then run the server binary.

## Licensing

This template itself is released under the Unlicense. You should replace the LICENSE for your own application with an appropriate license if you plan to release it publicly.
