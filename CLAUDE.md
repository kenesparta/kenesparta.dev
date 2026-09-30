# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a personal portfolio website built with Leptos (Rust full-stack web framework) using Axum as the backend
server. It runs as a Docker container on a shared AWS Lightsail **instance** (Ubuntu 24.04) behind CloudFront and
Caddy, with the image published to **GHCR**. Deployment is pull-based: a systemd timer on the host polls GHCR and
recreates the container when `latest` moves — CI never touches AWS. Blog posts are Markdown files in `content/posts/`
ingested into a **self-hosted PostgreSQL 18 container** on the same host.

**All infrastructure lives in the sibling repo `../personal-infra`** — Terraform for the AWS edge (Route 53, ACM,
CloudFront, the instance, the backup bucket) and Ansible for everything on the host (Docker, Caddy, Postgres, deploy
timers, backups, hardening). Its `spec/` directory is the authoritative record of every infra decision; read it before
proposing infra changes, and make them there, not here. This repo builds and ships the application image; nothing
more. The old `tf/` directory (Lightsail Container Service, ECR, per-repo CloudFront) was deleted in the migration —
do not recreate it.

**Security ledger:** `SECURITY.md` records every security-relevant change as a `SEC-NNN` entry (finding, change,
verification, follow-ups) plus the open findings of the last audit. Add an entry whenever you touch secrets handling,
the Makefile `blog/*`/`secrets*` targets, the workflows, CI trust (action pins, tokens), response headers, or bump a
dependency for an advisory — and read it before changing any of those.

**Tech Stack:**
- **Frontend/Backend**: Leptos 0.8.0 (full-stack Rust framework with SSR and hydration)
- **Web Server**: Axum 0.8.0
- **Styling**: plain CSS — source in `style/parts/*.css`, concatenated by `make css` into the generated
  `style/main.css` bundle (no Sass); cargo-leptos minifies via lightningcss
- **Logging**: `tracing` + `tracing-subscriber` emitting one JSON object per line on stdout
  (`telemetry::init()`, called first thing by both bins), filtered by `RUST_LOG` (default `info`).
  In production stdout ships to CloudWatch Logs (`/kenesparta/blog`, 7-day retention — personal-infra
  AD-11), where Logs Insights auto-parses the JSON fields. `telemetry::access_log` (outermost
  middleware) emits one INFO `"request"` event per page request — viewer IP + country/region/city,
  method, public path, status, latency, user-agent, referer — skipping `/pkg/*` and favicon noise.
  The IP arrives as `true-client-ip` (injected, spoof-proof, by a viewer-request CloudFront Function)
  and the geo as `CloudFront-Viewer-*` headers (forwarded by a custom cache policy) — both from
  personal-infra AD-12; locally they're absent and log as `-`. Geo values are percent-decoded;
  city/region are best-effort and may be `-` even in prod. `TraceLayer` (tower-http) additionally
  spans every request; its per-request events are DEBUG, so prod (`info`) adds only startup, errors
  and 5xx failures while dev (`RUST_LOG=debug` in compose) shows spans and SQLx queries. Log with
  fields (`tracing::info!(slug = %slug, "upserted")`), never interpolation; never log `DATABASE_URL`
  or raw header maps
- **Testing**: Playwright (end-to-end tests)
- **Architecture**: DDD / hexagonal — a Cargo workspace with a library crate per Bounded Context (`crates/bc-*`), a
  `shared-kernel`, and a single binary (`apps/backend`) that hosts the Leptos app plus all adapters and wiring
- **Database**: self-hosted `postgres:18` container (managed by personal-infra) on the instance's internal `web`
  Docker network. The app reaches it as `postgres:5432` via Docker DNS — never `127.0.0.1`, which inside a
  bridge-networked container is the container's own loopback. Access via SQLx — runtime `query_as` (no `query!`
  macros, so no DATABASE_URL at build time); migrations in `apps/backend/migrations/` embedded via `sqlx::migrate!()`
  and run at startup
- **Secrets**: sops + age — `secrets/{dev,prod}.enc.env` (dotenv, committed encrypted), recipients in `.sops.yaml`,
  age key at `~/.config/sops/age/keys.txt` (`SOPS_AGE_KEY_FILE` exported by the Makefile). These files only feed
  local tooling (dev server, ingest CLI) — the production container's env comes from personal-infra's Ansible Vault,
  not from here
- **Containerization**: Docker (multi-stage build → distroless runtime)
- **CI/CD**: GitHub Actions (`publish-image.yml`) — on `vX.Y.Z` tags, builds the image and pushes
  `ghcr.io/kenesparta/kenespartadev:vX.Y.Z` + `:latest` using only the repo's `GITHUB_TOKEN`. That is the whole
  pipeline; the host's timer does the rollout

## Repository Structure

```
.
├── Cargo.toml             # [workspace] — edition 2024, rust 1.98, shared deps
├── rust-toolchain.toml    # pins channel 1.98 + wasm32 target
├── clippy.toml · rustfmt.toml
├── crates/
│   ├── shared-kernel/     # Cross-cutting types: DomainError, Datetime, PostUuid
│   └── bc-blog/           # Bounded Context: blog (no runtime/IO deps)
│       └── src/
│           ├── domain/        # model.rs (BlogPost/Summary/PostStatus), repository.rs (BlogRepository port + upsert), errors.rs
│           └── application/   # use_cases.rs (List/GetBySlug/GetById/Upsert/Prune), dto.rs (BlogPostDTO…)
├── content/
│   └── posts/             # Blog posts: Markdown + TOML frontmatter (source of truth)
├── secrets/               # sops/age-encrypted dotenv files (committed; DATABASE_URL for local tooling)
├── .sops.yaml             # sops creation rules (age recipients)
├── apps/
│   └── backend/           # The Leptos app (SSR bin + hydrate lib) + all adapters
│       ├── src/
│       │   ├── main.rs · lib.rs           # server + wasm entry points
│       │   ├── bin/ingest.rs              # ingest CLI (feature `ingest`): markdown → HTML → upsert
│       │   ├── configuration.rs           # env config (DATABASE_URL, required — fails fast)
│       │   ├── composition.rs             # DI Container (PgPool + migrations → use cases)
│       │   ├── http.rs                    # ServerState + server-fn handler
│       │   ├── seo.rs                     # crawler endpoints: /sitemap.xml, /feed.xml, /llms.txt,
│       │   │                              #   /blog/<slug>.md + its rewrite middleware (ssr-only)
│       │   ├── security.rs                # response headers + per-request CSP nonce (SEC-004)
│       │   ├── telemetry.rs               # JSON tracing subscriber, shared by server + ingest bins
│       │   ├── persistence/blog_postgres.rs   # PostgresBlogRepository (implements the port)
│       │   └── app/                       # UI: app.rs (routing/shell), components/, pages/, constants.rs, api.rs (server fns)
│       ├── migrations/       # SQLx migrations (embedded into the binary)
│       ├── style/            # parts/*.css (source) → main.css (generated by `make css`)
│       ├── public/           # Static assets (incl. robots.txt)
│       ├── end2end/          # Playwright tests
│       ├── Cargo.toml        # Leptos config (output-name, site-root, bin-target, …)
│       └── ...
├── Dockerfile             # Multi-stage build (builds from apps/backend, distroless runtime)
├── .github/workflows/     # publish-image.yml (GHCR) + audit.yml
└── Makefile               # Build shortcuts (incl. secrets* and blog/* targets)
```

## Development Commands

### Local Development

**Prerequisites:**
- Rust toolchain per `rust-toolchain.toml` (1.98 + wasm32 target)
- cargo-leptos: `cargo install cargo-leptos --locked`
- Playwright deps (for tests): `cd apps/backend/end2end && pnpm install`

**Running the development server:**
```bash
make dev/up          # docker compose: app + PostgreSQL, DATABASE_URL wired
```
Or on the host (needs the dev DB: `docker compose -f docker-compose.dev.yml up -d postgres`):
```bash
sops exec-env secrets/dev.enc.env 'sh -c "cd apps/backend && cargo leptos watch"'
```
This starts the dev server with hot-reload at http://0.0.0.0:3000. The app
requires `DATABASE_URL` (no default — it fails fast without it).

**Secrets (sops + age):**
```bash
make secrets              # edit secrets/dev.enc.env in $EDITOR
make secrets-prod         # edit secrets/prod.enc.env
make secrets-view ENV=prod
make secrets-rotate       # after changing recipients in .sops.yaml
```
The age private key lives at `~/.config/sops/age/keys.txt` (never in the repo);
the Makefile exports `SOPS_AGE_KEY_FILE` because sops' macOS default path differs.

**Blog authoring (markdown → Postgres):**
```bash
make blog/ingest          # upsert content/posts/*.md into the dev DB
make blog/publish         # upsert into PRODUCTION (via SSH tunnel, see below)
```
Posts use TOML frontmatter between `+++` fences (title, summary, date RFC 3339,
optional slug/author/tags/status; status defaults to "draft"). The ingest CLI
(`apps/backend/src/bin/ingest.rs`, feature `ingest`) renders markdown with
pulldown-cmark and upserts by slug — idempotent, `post_id`/`created_at` preserved.
Each post is stored twice: `content` (rendered HTML, what the pages display) and
`content_md` (the body verbatim, what `/blog/<slug>.md` serves to agents). Posts
ingested before `content_md` existed have it empty and 404 on the `.md` URL until
re-ingested — after deploying that migration, re-run `make blog/publish`.
Deleting a `.md` file does NOT delete its DB row; pass `PRUNE=1` (→ `--prune`)
to also delete DB posts with no matching file, making the DB mirror `content/posts/`.

**How `blog/publish` reaches production:** the production Postgres has **no published
port** (deliberate — personal-infra acceptance criteria 8/9), so the target opens an
SSH tunnel first: it asks the host (`ubuntu@origin.kenesparta.dev`, key
`~/.ssh/personal-infra` — same as personal-infra's `ansible.cfg`) for the postgres
container's IP on the `web` Docker network (`docker inspect`; the IP can change across
restarts, so it is fetched every run), forwards `127.0.0.1:5433` to it via a
control-master ssh that a `trap` always tears down, and runs the ingest with the
`DATABASE_URL` host rewritten `@postgres:5432` → `@127.0.0.1:5433` on the fly.
The ingest binary is compiled **before** the tunnel opens and with no secret in its
environment (`make blog/build`: `env -u DATABASE_URL cargo build --locked …`), then
executed as `target/debug/ingest` under `sops exec-env`. Never fold that back into a
`cargo run` under sops: it would hand the production `DATABASE_URL` to every `build.rs`
and proc-macro in the dependency graph while the tunnel is open (SECURITY.md, SEC-001).
`secrets/prod.enc.env` therefore keeps the **canonical in-network URL**
(`…@postgres:5432/blog`) — do not point it at the tunnel. Override
`PUBLISH_SSH_KEY` / `PUBLISH_SSH_HOST` / `TUNNEL_PORT` if those defaults move.

**Building for production:**
```bash
cd apps/backend
cargo leptos build --release
```
Output: `target/release/backend` (binary) and `target/kdevsite/` (site assets)

**Type-checking the workspace (fast, no cargo-leptos):**
```bash
cargo check -p backend --features ssr                                     # server build
cargo check -p backend --features hydrate --target wasm32-unknown-unknown # wasm build
```

**Running end-to-end tests:**
```bash
cd apps/backend
cargo leptos end-to-end          # Debug mode
cargo leptos end-to-end --release # Release mode
```

### Compile Times

Source: *How to decrease your Rust compile times by 50%* (Let's Get Rusty,
https://www.youtube.com/watch?v=vFp4IbC2aZ0). `cargo build` runs in three stages — the
front end (parse, type check, borrow check), code generation (LLVM lowers the IR into
object files), then linking. Each optimization below targets a different stage, and the
video measures ~47% off a clean build and ~46% off a rebuild with all three stacked.

**1. Less debug information — applied, stable, no caveats.** Already in the root
`Cargo.toml`:

```toml
[profile.dev]
debug = "line-tables-only"
```

Cargo defaults `dev` to `debug = true`; emitting full DWARF and linking it into every
artifact is the largest avoidable cost of a debug build. `line-tables-only` keeps file
names and line numbers, so panics still name the right source line and backtraces stay
readable — the only loss is variable inspection in a step-through debugger session.
The video measures ~9% off a clean build and ~21% off a rebuild. This is the only one of
the three that runs on the pinned stable toolchain, which is why it is the one enabled.

**2. Parallel front end — nightly, not enabled.** The front end has historically run on
a single core regardless of how many the machine has (codegen was already parallel
across codegen units). `-Zthreads` parallelizes it:

```toml
# .cargo/config.toml
[build]
rustflags = ["-Zthreads=8"]
```

The video settles on 8 threads as the speed/memory balance; stacked on #1 it measures
~33% off both clean builds and rebuilds, and it helps CI clean builds as much as local
ones. `-Z` flags are nightly-only while `rust-toolchain.toml` pins 1.98 stable, so this
needs `cargo +nightly` — and because cargo-leptos shells out to cargo, the override has
to reach that child process (`RUSTUP_TOOLCHAIN=nightly cargo leptos watch`). Unlike #3
this flag is target-agnostic, so it is safe for the wasm artifact.

**3. Cranelift code generation — nightly, NOT usable here as written.** Cranelift
optimizes less than LLVM and emits machine code faster, which is the right trade for a
dev build:

```toml
# .cargo/config.toml
# first: rustup component add rustc-codegen-cranelift-preview --toolchain nightly
[unstable]
codegen-backend = true            # unstable flag gating the profile key below

[profile.dev]
codegen-backend = "cranelift"     # dev only — release stays on LLVM
```

The catch for this repo: cargo-leptos also builds a `wasm32-unknown-unknown` hydrate
artifact, and cranelift's support matrix has no wasm32 target. `[profile.dev]` covers
both the SSR bin and the hydrate lib, so setting it there breaks `cargo leptos watch`.
Enabling it would mean scoping cranelift to the server half — point cargo-leptos's
`bin-profile-dev` at a cranelift-enabled profile and leave `lib-profile-dev` on the
stock `dev` profile. Two further caveats: cranelift can fail outright on code LLVM
accepts (usually low-level CPU intrinsics — fall back to LLVM for that build), and on
macOS it does not support unwinding, so it forces `-Cpanic=abort`.

### Docker

**Building the Docker image:**
```bash
docker build -t kenespartadev .   # build context is the workspace root
```

The root `docker-compose.yml` (and `make prod/run`) exists only to smoke-test the
production image locally. The host does NOT use it — the compose file that actually
runs in production is rendered by personal-infra's Ansible.

### Infrastructure (../personal-infra)

Provisioning and host configuration are not done from this repo. In `../personal-infra`:
`make login` (AWS SSO) and Terraform plan/apply for the AWS edge; `make configure`
(Ansible `site.yml`) for host config; `make harden` deliberately separate. Adding or
changing a service is an edit to its `projects.yml` — one file read by both Terraform
and Ansible. Read its `spec/` (and `CLAUDE.md`) first; it records decisions and
rejected alternatives that are not obvious from the code.

## Architecture Notes

### Leptos Application Structure

The application uses Leptos's full-stack architecture with two compilation targets:
- **Server (SSR)**: Compiled with `ssr` feature, runs on Axum server
- **Client (WASM)**: Compiled with `hydrate` feature, runs in browser

**DDD layering (data flow):**
UI (`app/pages`, `app/components`) → server functions (`app/api.rs`) → use cases (`bc-blog/application`) via the DI
`Container` from Leptos context → `BlogRepository` port → `PostgresBlogRepository` adapter
(`persistence/blog_postgres.rs`, SQLx). The Bounded Context (`crates/bc-blog`) has no runtime/HTTP/SQLx dependencies;
those live only in `apps/backend`. `bc-blog` is compiled for both wasm and SSR so the UI can use its DTOs directly.
The write path (`upsert`, keyed by slug) exists only for the ingest CLI; the web app never writes.

**Routing:**
Routes are defined in `apps/backend/src/app.rs` using leptos_router:
- `/` → HomePage
- `/about` → About
- `/blog` → BlogList (`SsrMode::Async`)
- `/blog/:slug` → BlogPost (`SsrMode::Async`)
- `/experience` → Experience
- `/projects` → Projects
- unmatched → NotFound (real HTTP 404; unknown blog slugs also 404)

The blog routes use `SsrMode::Async` on purpose: the default out-of-order streaming ships a fallback plus an inert
`<template>` swapped in by script, which is invisible to crawlers that do not execute JavaScript. Every page mounts
one `PageMeta` component (`app/components/page_meta.rs`) — title, description, canonical, Open Graph — so no two URLs
share metadata. Crawler endpoints live outside the Leptos router: `/sitemap.xml`, `/feed.xml`, `/llms.txt` and the
`/blog/<slug>.md` Markdown variants in `src/seo.rs` (Axum handlers, ssr-only), `robots.txt` in `public/`.

`/llms.txt` follows llmstxt.org (H1, blockquote, H2 link lists) and links posts by their `.md` URL, so an agent
following a link gets the authored Markdown instead of hydrated HTML. That variant is served from the `content_md`
column (see *Blog authoring*); drafts and rows with empty `content_md` 404. Its URL needs a dynamic segment with a
literal `.md` suffix, which matchit (axum's router) cannot express — "dynamic suffixes are not currently supported" —
and `/blog/{slug}` is already the Leptos page route. So `seo::rewrite_markdown_suffix` rewrites `/blog/<slug>.md` to
the internal `/blog-md/{slug}` **before** routing: it is layered on an outer `Router` that holds the real router as
its `fallback_service`, because `Router::layer` runs *after* the match and would be too late. `robots.txt` disallows
`/blog-md/` so the internal path is not indexed as a duplicate.

The navigation bar (StickyNavBar) is conditionally rendered on all pages except the home page.

**Components:**
- Components are in `apps/backend/src/app/components/` (header, social links, navigation, blog, page_meta)
- Pages are in `apps/backend/src/app/pages/` (individual route handlers)

**Styling:**
- Edit styles in `apps/backend/style/parts/*.css` (ordered by numeric prefix), then run `make css` to concatenate
  them into the generated `apps/backend/style/main.css` bundle that cargo-leptos serves. `main.css` is gitignored;
  Docker builds regenerate it automatically. `make leptos/build` runs `make css` first.
- Leptos config specifies: `style-file = "style/main.css"`
- Compiled CSS is served at `/pkg/kenespartadev.css`

### Docker Deployment

Multi-stage Dockerfile (build context = workspace root):
1. **Builder stage**: Uses `rust:1.98-bookworm`, installs cargo-leptos, `COPY . .`, builds the CSS bundle, runs
   `cd apps/backend && cargo leptos build --release`
2. **Runtime stage**: Uses distroless image, copies the `backend` binary + `kdevsite/` site assets, runs as non-root

Environment variables for production:
- `LEPTOS_OUTPUT_NAME=kenespartadev`, `LEPTOS_SITE_ADDR="0.0.0.0:3000"`, `LEPTOS_SITE_ROOT=/app/kdevsite`,
  `LEPTOS_SITE_PKG_DIR=pkg`, `RUST_LOG` — baked into the image / set per project in personal-infra's `projects.yml`
- `DATABASE_URL` — injected at runtime from the host's root-owned `.env` (rendered by Ansible from its Vault);
  never baked into the image and never sourced from this repo's `secrets/`

Build args (compile time, not runtime env):
- `APP_VERSION` / `APP_BUILD` — the release tag and commit SHA, passed by `publish-image.yml` and read by
  `option_env!` in `app/constants.rs` for the home-page footer (`v0.5.3 · build 17b2c7e`). Compiled into the
  server binary and the wasm bundle alike, so hydration matches. `.git/` is not in the build context, so there is
  no other way to learn them; local and dev builds leave them unset and render `dev` (SECURITY.md SEC-017)

### Production Topology (personal-infra)

Request path: `kenesparta.dev` (Route 53 apex ALIAS) → CloudFront (ACM cert, `Managed-CachingDisabled` — pure
pass-through so SSR responses stay fresh) → `origin.kenesparta.dev` (A record to the instance's static IP) → Caddy
(Let's Encrypt cert; rejects any request missing the `X-Origin-Verify` header CloudFront injects) → `blog` container
(`blog:3000`) on the internal `web` Docker network. No container publishes host ports except Caddy; the instance is
shared with the other projects in personal-infra's `projects.yml`.

On the host, all managed by Ansible (hand edits are reverted):
- `/opt/personal-infra/projects/blog/` — the real `docker-compose.yml` + root-owned `.env` (0600)
- `personal-infra-deploy@blog.timer` — every 10 minutes runs `docker compose pull && up -d`. The compose file pins
  the deliberately **moving** `latest` tag: the pull is the release mechanism
- Nightly (03:00 UTC) `pg_dump` of every project database to `s3://kenesparta-infra-backups/postgres/<db>/`
- Database administration is `ssh` + `docker exec psql` — there is no network path to Postgres from outside

### CI/CD Pipeline

GitHub Actions (`.github/workflows/publish-image.yml`), on version tags:

```bash
git tag v1.0.0 -m v1.0.0 && git push origin v1.0.0   # signed: tag.gpgSign=true (SEC-008)
```

Release tags are annotated and GPG-signed (`tag.gpgSign` is set in this repo's git
config — the `-m` is required, a bare `git tag vX.Y.Z` opens the editor), and a
GitHub tag ruleset (`protect-release-tags`) restricts creating/moving/deleting
`v*` tags to repository admins. Pushing a `v*` tag IS the deploy, so those two
controls are the release gate.

builds the image and pushes `ghcr.io/kenesparta/kenespartadev:vX.Y.Z` + `:latest` to GHCR, authenticated with the
repo's own `GITHUB_TOKEN` — no AWS credentials, no OIDC role, no Terraform. The host's deploy timer picks up the
moved `latest` within ~10 minutes; there is nothing to watch in AWS. `audit.yml` runs dependency audits.

**No build cache, deliberately.** The workflow carried `cache-from`/`cache-to: type=gha` until it was measured on
the v0.4.1 run: ~205s per build (40% of an 8m29s run, 87.5s of it just "preparing build cache for export") spent
writing 3.2 GB, for **zero** cache hits — the log showed all 525 `Compiling` lines either way. The cause is scoping.
Actions caches are keyed per git ref, and this workflow only triggers on tags, so every run wrote its cache under
`refs/tags/vX.Y.Z` — a scope the next tag's run can never restore from. It was a write-only cache by construction.
Removing it takes the build to ~5m with nothing lost.

Fixing only the backend would not buy much either. `type=registry` cache is not ref-scoped, so it would at least
persist across tags, but `COPY . .` in the Dockerfile sits above `RUN cargo leptos build`: every commit invalidates
the compile layer, and the only restorable layers left are `rustup target add` and the cargo-leptos install, ~7s
combined. A cache is worth reinstating only alongside a cacheable dependency layer (`cargo-chef`) that keys the
~500 third-party crates on `Cargo.lock` — which has to cook twice here, once for the native `release` build and once
for `wasm32-unknown-unknown` / `wasm-release`.

**Where the time goes** (v0.4.1, 8m29s total, before the cache removal):
- `cargo leptos build --release` — 286s: 212s the native SSR `release` build, 73s the wasm hydrate artifact
- exporting to GitHub Actions Cache — 205s (now removed)
- `rustup target add wasm32`, the cargo-leptos install and the GHCR push — ~13s combined

The `debug = "line-tables-only"` win under *Compile Times* does not apply to CI, which builds `--release` with debug
info already off. Of those three optimizations only `-Zthreads` would bite into the 212s native build, at the cost
of putting nightly in the release image.

**Cost:** the $12/mo Lightsail instance is shared across all personal-infra projects; CloudFront, S3 and the backup
bucket are pay-per-use. No ECR, no managed database.

## Cargo.toml Configuration

Leptos package metadata lives in `apps/backend/Cargo.toml`:
- `output-name = "kenespartadev"`
- `site-root = "target/kdevsite"`
- `site-addr = "0.0.0.0:3000"`
- `reload-port = 3001` (for hot-reload)
- `end2end-cmd = "pnpm exec playwright test"`

## Testing

Playwright tests are located in `apps/backend/end2end/tests/`.

Test configuration in `apps/backend/end2end/playwright.config.ts`:
- Runs tests in parallel (chromium, firefox, webkit)
- 30s timeout per test
- HTML reporter
