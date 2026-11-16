# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a personal portfolio website built with Leptos (Rust full-stack web framework) using Axum as the backend
server. It runs as the AWS **Lambda function `blog`** in the account `personal-infra-prod`, behind CloudFront, from
an image published to a private **ECR** repository (`kenespartadev`); the AWS Lambda Web Adapter inside the image
replays each invocation as an HTTP request to the unchanged server (personal-infra rev 2.33, AD-18). CI's only AWS
access is an OIDC role, from `v*` tags only, that can push to that one repository and point that one function at the
pushed digest. Blog posts are Markdown files in `content/posts/` ingested into **Aurora DSQL** (personal-infra
AD-19), reached with IAM tokens: there is no database password anywhere.

Until personal-infra's cutover (its N9) the public name still reaches the old Lightsail host, which runs the last
image the old account received, from its own Postgres; it gets no new release (its G63).

**All infrastructure lives in the sibling repo `../personal-infra`** — Terraform for everything AWS: Route 53, ACM,
CloudFront, the Lambda functions and their URLs, the Aurora DSQL clusters and their backups, ECR and the CI roles.
Its old host (Ansible, Caddy, Postgres) lives on only in its worktree of `pre-account-migration` until its N13. Its `spec/` directory is the authoritative record of every infra decision; read it before
proposing infra changes, and make them there, not here. This repo builds and ships the application image; nothing
more. The old `tf/` directory (Lightsail Container Service, ECR, per-repo CloudFront) was deleted in the migration —
do not recreate it. Today's ECR repository and its push role are personal-infra's (spec AD-15), not a revival of it.

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
- **Testing**: Playwright for Python (`pytest-playwright`, end-to-end tests), managed by uv — no Node toolchain
- **Architecture**: DDD / hexagonal — a Cargo workspace with a library crate per Bounded Context (`crates/bc-*`), a
  `shared-kernel`, and a single binary (`apps/backend`) that hosts the Leptos app plus all adapters and wiring
- **Database**: in production, **Aurora DSQL** (cluster `blog`, schema `blog`), a PostgreSQL 16 dialect with no
  passwords: `dsql.rs` mints an IAM token per connection with the function's role and connects as `blog_app`, which
  may only `SELECT` (personal-infra §5.22). Locally, a plain `postgres` container (`DATABASE_URL`). `Configuration`
  picks DSQL when `DSQL_ENDPOINT` is set. Access via SQLx — runtime `query_as` (no `query!` macros, so no
  DATABASE_URL at build time). Migrations in `apps/backend/migrations/` are embedded via `sqlx::migrate!()` and run
  **only by `ingest`** (`migrations::run`, locking off), never at server startup. Every migration is written for DSQL
  and still valid on Postgres: first line `-- no-transaction`, exactly one DDL statement, no arrays (`tags` is
  `jsonb`), no `ADD COLUMN` with a constraint, no plain `CREATE INDEX` (personal-infra G51). Never edit an applied
  migration, comments included: sqlx checks each file's checksum and refuses to run past a changed one
- **Secrets**: sops + age — `secrets/{dev,prod}.enc.env` (dotenv, committed encrypted), recipients in `.sops.yaml`,
  age key at `~/.config/sops/age/keys.txt` (`SOPS_AGE_KEY_FILE` exported by the Makefile). These files only feed
  local tooling (dev server, local ingest). Production has no database secret at all, and the function's only
  secret, `ORIGIN_SECRET`, comes from personal-infra's Terraform
- **Containerization**: Docker (multi-stage build → distroless runtime)
- **CI/CD**: GitHub Actions (`publish-image.yml`) — on `vX.Y.Z` tags, assumes the OIDC role
  `github-actions-ecr-push-kenespartadev`, builds the image (`provenance: false`), pushes
  `<registry>/kenespartadev:vX.Y.Z` + `:latest` to private ECR, then runs `aws lambda update-function-code` with the
  pushed digest. That last step is the deploy

## Repository Structure

```
.
├── Cargo.toml             # [workspace] — edition 2024, rust 1.100, shared deps
├── rust-toolchain.toml    # pins channel 1.100 + wasm32 target
├── clippy.toml · rustfmt.toml
├── crates/
│   ├── shared-kernel/     # Cross-cutting types: DomainError, Datetime, PostUuid
│   └── bc-blog/           # Bounded Context: blog (no runtime/IO deps)
│       └── src/
│           ├── domain/        # model.rs (BlogPost/Summary/PostStatus), repository.rs (BlogRepository port + upsert), errors.rs
│           └── application/   # use_cases.rs (List/GetBySlug/GetById/Upsert/Prune), dto.rs (BlogPostDTO…)
├── content/
│   └── posts/             # Blog posts: Markdown + TOML frontmatter (source of truth)
├── secrets/               # sops/age-encrypted dotenv files (committed; DATABASE_URL for local tooling only)
├── .sops.yaml             # sops creation rules (age recipients)
├── apps/
│   └── backend/           # The Leptos app (SSR bin + hydrate lib) + all adapters
│       ├── src/
│       │   ├── main.rs · lib.rs           # server + wasm entry points
│       │   ├── bin/ingest.rs              # ingest CLI (feature `ingest`): markdown → HTML → upsert
│       │   ├── configuration.rs           # env config: DATABASE_URL locally, DSQL_* in production (fails fast)
│       │   ├── composition.rs             # connect (pool, DSQL refresher) + wire (use cases); never migrates
│       │   ├── caching.rs                 # Cache-Control for files from disk: hashed /pkg a year, public/ a day
│       │   ├── dsql.rs                    # Aurora DSQL: IAM token per connection, refresh before each request
│       │   ├── migrations.rs              # runs the embedded migrations, lock off — ingest only
│       │   ├── origin_gate.rs             # 403 without CloudFront's X-Origin-Verify (SEC-025)
│       │   ├── http.rs                    # ServerState + server-fn handler
│       │   ├── seo.rs                     # crawler endpoints: /sitemap.xml, /feed.xml, /llms.txt,
│       │   │                              #   /blog/<slug>.md + its rewrite middleware (ssr-only)
│       │   ├── security.rs                # response headers + per-request CSP nonce (SEC-004)
│       │   ├── telemetry.rs               # JSON tracing subscriber, shared by server + ingest bins
│       │   ├── persistence/blog_postgres.rs   # PostgresBlogRepository (implements the port)
│       │   └── app/                       # UI: app.rs (routing/shell), components/, pages/, constants.rs, api.rs (server fns)
│       ├── migrations/       # SQLx migrations, written for DSQL (embedded; one DDL each, `-- no-transaction`)
│       ├── style/            # parts/*.css (source) → main.css (generated by `make css`)
│       ├── public/           # Static assets (incl. robots.txt)
│       ├── end2end/          # Playwright tests: pytest + uv (pyproject.toml, uv.lock, tests/test_*.py)
│       ├── Cargo.toml        # Leptos config (output-name, site-root, bin-target, …)
│       └── ...
├── Dockerfile             # Multi-stage build (builds from apps/backend, distroless runtime)
├── .github/workflows/     # publish-image.yml (ECR) + audit.yml
└── Makefile               # Build shortcuts (incl. secrets* and blog/* targets)
```

## Development Commands

### Local Development

**Prerequisites:**
- Rust toolchain per `rust-toolchain.toml` (1.100 + wasm32 target)
- cargo-leptos: `cargo install cargo-leptos --locked`
- uv (for the e2e tests); browser once per Playwright version:
  `cd apps/backend/end2end && uv run --locked playwright install chromium`

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
make blog/ingest          # migrate + upsert content/posts/*.md into the dev DB
make blog/publish         # migrate + upsert into PRODUCTION's Aurora DSQL (IAM, see below)
```
Posts use TOML frontmatter between `+++` fences (title, summary, date RFC 3339,
optional slug/author/tags/status; status defaults to "draft"). The ingest CLI
(`apps/backend/src/bin/ingest.rs`, feature `ingest`) renders markdown with
pulldown-cmark and upserts by slug — idempotent, `post_id`/`created_at` preserved.
Each post is stored twice: `content` (rendered HTML, what the pages display) and
`content_md` (the body verbatim, what `/blog/<slug>.md` serves to agents).
`ingest` applies pending migrations before it upserts, and it is the only thing that
does: the web server never migrates. So a fresh local database needs one
`make blog/ingest` before the dev server can list anything, and a release whose code
needs a new migration needs `make blog/publish` **before** its tag. A local database
created before the DSQL baseline (2026-11-16) must be recreated:
`docker compose -f docker-compose.dev.yml rm -s -f postgres && docker volume rm kenespartadev_pg-data`, then
`docker compose -f docker-compose.dev.yml up -d postgres` and `make blog/ingest`. Never `down -v`: it
also deletes the dev container's cargo and target caches.
Deleting a `.md` file does NOT delete its DB row; pass `PRUNE=1` (→ `--prune`)
to also delete DB posts with no matching file, making the DB mirror `content/posts/`.

**How `blog/publish` reaches production:** directly, over TLS, with an IAM token and no
secret (SECURITY.md SEC-025). The target reads two things from `../personal-infra`
(`PUBLISH_INFRA`), the repository that owns them: the SSO profile in its
`terraform/.env` and the `blog` cluster endpoint in its `dsql_endpoints` output. It then
runs `ingest` as **`blog_owner`**, the schema's owner, to which personal-infra's
`make dsql/roles` maps both your SSO role and CI's push role. DSQL has no `SET ROLE`, so
connecting as the role is the only way to own what a migration creates: a table created
by `blog_owner` inherits the grant that lets the function's `blog_app` read it.
`blog/build` still compiles the binary first with no secret in its environment (SEC-001),
and the run unsets `DATABASE_URL`. Until personal-infra's N9 the public site reads the
old host's Postgres, so a post published here is live only on the new function.

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
Both start the server (they need `DATABASE_URL`, like the dev server) and then run `end2end-cmd`
(`uv run --locked pytest`) in `apps/backend/end2end/`. Against a server that is already up, run
`uv run --locked pytest` there directly; `--base-url` points it elsewhere.

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
ones. `-Z` flags are nightly-only while `rust-toolchain.toml` pins 1.100 stable, so this
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
1. **Builder stage**: Uses `rust:1.100-bookworm`, installs cargo-leptos, `COPY . .`, builds the CSS bundle, runs
   `cargo leptos build --release --precompress` with `LEPTOS_HASH_FILES=true`: the bundle is named after its contents
   (`pkg/kenespartadev.<hash>.{css,js,wasm}`, the hashes in `hash.txt`) and every file gets a `.br`/`.gz` beside it
2. **Runtime stage**: Uses distroless image, copies the `backend` binary, `hash.txt` **beside it** (leptos reads it from
   the executable's directory) and `kdevsite/`, adds the Lambda Web Adapter, sets `LEPTOS_HASH_FILES=true`, runs as
   non-root. `caching.rs` then gives `/pkg/*` a year and `immutable`, which is also what lets personal-infra's edge
   cache it (its §5.16); `public/` gets a day, rendered pages nothing. Under `cargo leptos watch` the names are not
   hashed and `/pkg/*` is `no-cache`, so the two can never disagree

Environment variables for production:
- `LEPTOS_OUTPUT_NAME=kenespartadev`, `LEPTOS_SITE_ADDR="0.0.0.0:3000"`, `LEPTOS_SITE_ROOT=/app/kdevsite`,
  `LEPTOS_SITE_PKG_DIR=pkg`, `RUST_LOG` — baked into the image / set per project in personal-infra's `projects.yml`
- `DSQL_ENDPOINT`, `DSQL_USER` (`blog_app`), `DSQL_SCHEMA` (`blog`) and `ORIGIN_SECRET` — set on the function by
  personal-infra's Terraform. None is baked into the image; the only secret, `ORIGIN_SECRET`, never comes from this
  repo's `secrets/`
- `AWS_LWA_PORT` (3000), `AWS_LWA_READINESS_CHECK_PROTOCOL` (`tcp`), `AWS_LWA_INVOKE_MODE` (`response_stream`) —
  the Lambda Web Adapter's, from the same place. The adapter itself is copied into the image, pinned by digest

Build args (compile time, not runtime env):
- `APP_VERSION` / `APP_BUILD` — the release tag and commit SHA, passed by `publish-image.yml` and read by
  `option_env!` in `app/constants.rs` for the home-page footer (`v0.5.3 · build 17b2c7e`). Compiled into the
  server binary and the wasm bundle alike, so hydration matches. `.git/` is not in the build context, so there is
  no other way to learn them; local and dev builds leave them unset and render `dev` (SECURITY.md SEC-017)

### Production Topology (personal-infra)

Request path: `kenesparta.dev` (Route 53 apex ALIAS) → CloudFront (ACM cert, a caching-disabled policy that forwards
the geo headers) → the **Function URL** of `blog` → the Lambda Web Adapter → this server on `:3000`.
`origin_gate::check` answers 403 to any request without CloudFront's `X-Origin-Verify` header, because the Function
URL itself is public (SEC-025). That check was Caddy's on the old host. Lambda's own 403, with a JSON body, means a
missing permission rather than the gate (personal-infra G46).

The function holds no state. Data is in Aurora DSQL, backed up daily by AWS Backup with 35 days kept; logs go to the
CloudWatch group `/kenesparta/blog` through the function's `logging_config`. Database administration is
`make dsql/connect DB=blog` in personal-infra: an admin `psql` with an IAM token. Until personal-infra's N9 the old
host serves the public name, and its description is in that repository's worktree of `pre-account-migration`.

### CI/CD Pipeline

GitHub Actions (`.github/workflows/publish-image.yml`), on version tags:

```bash
git tag v1.0.0 -m v1.0.0 && git push origin v1.0.0   # signed: tag.gpgSign=true (SEC-008)
```

Release tags are annotated and GPG-signed (`tag.gpgSign` is set in this repo's git
config — the `-m` is required, a bare `git tag vX.Y.Z` opens the editor), and a
GitHub tag ruleset (`protect-release-tags`) restricts creating/moving/deleting
`v*` tags to repository admins. Pushing a `v*` tag IS the deploy, so those two
controls are the release gate — and, because the push role trusts only
`refs/tags/v*`, the gate on CI's AWS access too.

The workflow builds the image and pushes `<registry>/kenespartadev:vX.Y.Z` + `:latest` to the private ECR repository. It
authenticates by assuming `github-actions-ecr-push-kenespartadev` over GitHub OIDC — trusted only from this repo's
`refs/tags/v*`, allowed to push to that one repository and nothing else — so there is no stored AWS key; the only
repository secret is the role's ARN, `AWS_ECR_PUSH_ROLE_ARN` (set from `make output NAME=ecr_push_role_arns` in
personal-infra — the new account's role since its N5). The registry host is `amazon-ecr-login`'s `registry` output,
never a literal in this repo. No Terraform runs here. The build pushes a single manifest (`provenance: false`, since
Lambda refuses an index — personal-infra G49), and the "Deploy to Lambda" step points the function at the pushed
digest and waits; a function never pulls by itself (G48). `audit.yml` runs dependency audits.

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
- `rustup target add wasm32`, the cargo-leptos install and the registry push (GHCR, at the time) — ~13s combined

The `debug = "line-tables-only"` win under *Compile Times* does not apply to CI, which builds `--release` with debug
info already off. Of those three optimizations only `-Zthreads` would bite into the 212s native build, at the cost
of putting nightly in the release image.

**Cost:** the function and the DSQL cluster sit inside their free tiers at this traffic (personal-infra §14 rev 2.32
measured ~2,000 DPU a month against 100,000 free); CloudFront and ECR storage (the lifecycle keeps `latest` plus the
newest 30 images) are pay-per-use.

## Cargo.toml Configuration

Leptos package metadata lives in `apps/backend/Cargo.toml`:
- `output-name = "kenespartadev"`
- `site-root = "target/kdevsite"`
- `site-addr = "0.0.0.0:3000"`
- `reload-port = 3001` (for hot-reload)
- `end2end-cmd = "uv run --locked pytest"`
- `end2end-dir = "apps/backend/end2end"` — relative to the **workspace root**: cargo-leptos chdirs there
  before spawning `end2end-cmd`, so a package-relative `end2end` points nowhere

## Testing

Playwright tests are Python, in `apps/backend/end2end/tests/` (`test_*.py`, pytest-playwright's
sync API), with shared fixtures in `tests/conftest.py`. The suite is a uv project: dependencies in
`pyproject.toml`, exact versions and hashes in `uv.lock` (Dependabot's `uv` ecosystem bumps it),
Python pinned by `.python-version`. Always run it `--locked`, so a stale lockfile fails instead of
being re-resolved. There is no Node toolchain here: the `playwright` wheel bundles its own driver.

**Dependency hardening (SECURITY.md SEC-018) — keep all of it when touching `pyproject.toml`:**
- Direct dependencies are **exact pins** (`==`); transitive ones are pinned with per-file sha256 in
  `uv.lock`. Bump a pin and re-lock together (`uv lock`), never loosen to a range.
  `requires-python = "==3.14.*"` pins the minor and lets security patches float.
- `[tool.uv] no-build = true` — wheels only, so no sdist build backend (arbitrary code) ever runs.
  A new dependency that ships only an sdist fails the lock; do not lift the flag for it.
- `[tool.uv] exclude-newer = "7 days"` — the resolver ignores releases younger than a week (the
  SEC-003 week-old rule). Dependabot's `uv` entry has the matching `cooldown: default-days: 7`;
  change both or neither. A pin to a fresher release cannot lock until it ages past the window.
- `audit.yml`'s `uv-audit` job runs `uv audit --frozen` weekly and on every change to
  `pyproject.toml`/`uv.lock` (SEC-020), with uv pinned to an explicit version installed through the
  same SHA-pinned `taiki-e/install-action`. `uv audit` does not flag yanked releases, so check
  those by hand in the Monday Dependabot pass.

Configuration lives in `[tool.pytest.ini_options]` in `pyproject.toml`, and fixtures are overridden
in `conftest.py`, not in a config file:
- `base_url = "http://localhost:3000"` — browser tests use relative paths (`page.goto("/blog")`)
- `--tracing retain-on-failure` — a failing test leaves a trace in `test-results/`
  (`uv run playwright show-trace <zip>`)
- Chromium by default; add `--browser firefox --browser webkit` (after installing them) for the others
- The `api` fixture (`conftest.py`) is an `APIRequestContext` for header/redirect checks that need
  no browser. Requests whose path starts with `//` must use an absolute URL: relative to
  `base_url`, `//evil.com/` is protocol-relative and leaves the site
