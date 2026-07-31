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

**Tech Stack:**
- **Frontend/Backend**: Leptos 0.8.0 (full-stack Rust framework with SSR and hydration)
- **Web Server**: Axum 0.8.0
- **Styling**: plain CSS — source in `style/parts/*.css`, concatenated by `make css` into the generated
  `style/main.css` bundle (no Sass); cargo-leptos minifies via lightningcss
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
├── Cargo.toml             # [workspace] — edition 2024, rust 1.97, shared deps
├── rust-toolchain.toml    # pins channel 1.97 + wasm32 target
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
- Rust toolchain per `rust-toolchain.toml` (1.97 + wasm32 target)
- cargo-leptos: `cargo install cargo-leptos --locked`
- Playwright deps (for tests): `cd apps/backend/end2end && npm install`

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
1. **Builder stage**: Uses `rust:1.97-bookworm`, installs cargo-leptos, `COPY . .`, builds the CSS bundle, runs
   `cd apps/backend && cargo leptos build --release`
2. **Runtime stage**: Uses distroless image, copies the `backend` binary + `kdevsite/` site assets, runs as non-root

Environment variables for production:
- `LEPTOS_OUTPUT_NAME=kenespartadev`, `LEPTOS_SITE_ADDR="0.0.0.0:3000"`, `LEPTOS_SITE_ROOT=/app/kdevsite`,
  `LEPTOS_SITE_PKG_DIR=pkg`, `RUST_LOG` — baked into the image / set per project in personal-infra's `projects.yml`
- `DATABASE_URL` — injected at runtime from the host's root-owned `.env` (rendered by Ansible from its Vault);
  never baked into the image and never sourced from this repo's `secrets/`

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
git tag v1.0.0 && git push origin v1.0.0
```

builds the image and pushes `ghcr.io/kenesparta/kenespartadev:vX.Y.Z` + `:latest` to GHCR, authenticated with the
repo's own `GITHUB_TOKEN` — no AWS credentials, no OIDC role, no Terraform. The host's deploy timer picks up the
moved `latest` within ~10 minutes; there is nothing to watch in AWS. `audit.yml` runs dependency audits.

**Cost:** the $12/mo Lightsail instance is shared across all personal-infra projects; CloudFront, S3 and the backup
bucket are pay-per-use. No ECR, no managed database.

## Cargo.toml Configuration

Leptos package metadata lives in `apps/backend/Cargo.toml`:
- `output-name = "kenespartadev"`
- `site-root = "target/kdevsite"`
- `site-addr = "0.0.0.0:3000"`
- `reload-port = 3001` (for hot-reload)
- `end2end-cmd = "npx playwright test"`

## Testing

Playwright tests are located in `apps/backend/end2end/tests/`.

Test configuration in `apps/backend/end2end/playwright.config.ts`:
- Runs tests in parallel (chromium, firefox, webkit)
- 30s timeout per test
- HTML reporter
