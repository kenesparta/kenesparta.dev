+++
title = "Building kenesparta.dev: a Rust site from browser to database"
summary = "How this site is built — Leptos SSR with hydration, a hexagonal Cargo workspace, Markdown posts ingested into PostgreSQL with SQLx, and a $7/month deploy on AWS Lightsail behind CloudFront."
date = "2026-07-21T00:00:00Z"
tags = ["rust", "leptos", "axum", "sqlx", "postgres", "aws", "terraform", "docker"]
status = "draft"  # flip to "published" when the images are in
+++

This site is Rust all the way down: the pages you are reading were rendered by
a Rust binary, hydrated in your browser by the same Rust code compiled to
WebAssembly, and pulled from a PostgreSQL database by a Rust CLI-driven
pipeline. This post is a tour of how it fits together — the framework, the
architecture, the content pipeline, and the infrastructure — and why each
piece ended up the way it did.

<!-- TODO image: high-level architecture diagram
     https://cdn.kenesparta.dev/blog/building-kenesparta-dev/architecture.webp -->

## One language, two targets

The site is built with [Leptos](https://leptos.dev) 0.8, a full-stack Rust
web framework, served by [Axum](https://github.com/tokio-rs/axum) 0.8 on
Tokio. The toolchain is pinned — Rust 1.97, edition 2024, plus the
`wasm32-unknown-unknown` target — and `cargo-leptos` compiles the same
codebase twice:

- **Server (SSR)**: a native binary that renders every page to HTML, so the
  first paint needs no JavaScript at all.
- **Client (WASM)**: the same components compiled to WebAssembly, which
  *hydrate* the server-rendered HTML and take over interactivity.

There is no REST API layer to keep in sync. Leptos *server functions* let a
component call what looks like a plain async Rust function; on the server it
runs directly, and in the browser it becomes a typed RPC call. One function
signature, checked by the compiler on both sides.

Styling is deliberately boring: plain CSS files under `style/parts/`,
concatenated into a single bundle by `make css` and minified with
lightningcss. No Tailwind, no Sass, no build-time surprises. Two small
performance touches round it out: the server compresses responses with
`tower-http`, and the WASM artifact gets its own size-focused release
profile (`opt-level = "z"`, LTO, `panic = "abort"`, stripped).

## A hexagonal workspace

The repository is a Cargo workspace laid out along DDD lines:

- `crates/bc-blog` — the *blog* Bounded Context: domain model (`BlogPost`,
  `PostStatus`), use cases, and a `BlogRepository` port. It has **no** HTTP,
  SQL, or runtime dependencies, so it compiles unchanged for both the server
  and the WASM client.
- `crates/shared-kernel` — small cross-cutting types (IDs, datetimes, domain
  errors).
- `apps/backend` — the only binary, with `unsafe_code = "forbid"`. It hosts
  the Leptos app and every adapter: the Axum server, the SQLx implementation
  of `BlogRepository`, configuration, and the dependency-injection wiring.

The flow for a page like `/blog/{slug}` is:

```
UI component → server function → use case (bc-blog) → repository port
                                                        └─ Postgres adapter (SQLx)
```

Is this over-engineered for a personal site? Absolutely — and that's the
point. The site is a small, low-stakes place to practice the architecture I
want to be fluent in, with the compiler enforcing the boundaries. From the
outside, Playwright end-to-end tests exercise the rendered site in Chromium,
Firefox, and WebKit.

## The data layer: SQLx and PostgreSQL on Lightsail

The blog lives in a Lightsail **managed PostgreSQL** instance, and the app
talks to it through [SQLx](https://github.com/launchbadge/sqlx) — plain SQL,
no ORM. Two deliberate choices shape that integration:

- **Runtime queries, not compile-time macros.** Everything goes through
  `query_as` with `FromRow`, never the `query!` macros. The trade-off is
  losing compile-time SQL checking; the win is that building the app needs
  no `DATABASE_URL` or offline cache — the Docker image compiles anywhere,
  and credentials exist only at runtime.
- **Migrations embedded in the binary.** `sqlx::migrate!()` compiles the
  `.sql` files into the executable and runs them at startup, so a fresh
  database bootstraps itself. Migration files are checksummed and
  append-only: an applied migration is never edited, only followed by a new
  one.

The database itself is the one piece created outside Terraform — the data
should outlive any infrastructure experiment. It keeps public access
switched off, so only Lightsail resources in the region can reach it;
publishing from my laptop means toggling public mode on for one command and
straight back off.

## Posts are Markdown, the database is a cache of the repo

Every post is a Markdown file with TOML frontmatter (title, summary,
RFC 3339 date, tags, status) under `content/posts/`, version-controlled with
the rest of the site. Publishing is one command:

```sh
make blog/publish
```

An ingest CLI parses the frontmatter, renders the Markdown with
`pulldown-cmark` (tables, footnotes, strikethrough), and *upserts* each post
into PostgreSQL, keyed by slug — which defaults to the file name. Re-running
it is idempotent: `post_id` and `created_at` are preserved, so URLs never
break and edits are just re-ingests. Posts default to `draft` status —
nothing goes public by omission — and a `--prune` flag deletes database rows
whose file is gone, making the database exactly mirror the repo (with a
guard: an empty content directory never prunes anything).

The CLI is a second binary in the same crate, feature-gated so it reuses the
server's exact composition — pool, migrations, use cases — while staying out
of the production image. The web app never writes to the database at all;
ingest is the only write path.

<!-- TODO image: screenshot of `make blog/publish` output
     https://cdn.kenesparta.dev/blog/building-kenesparta-dev/publish.webp -->

## Secrets live in the repo — encrypted

Database credentials sit in `secrets/{dev,prod}.enc.env`, committed to the
repository but encrypted with [sops](https://github.com/getsops/sops) and
[age](https://age-encryption.org). Local commands run under
`sops exec-env`, and Terraform's sops provider decrypts the production file
at apply time to inject `DATABASE_URL` into the container — production runs
with the same file everything else reads, so there is no drift. The age
private key exists in exactly two places: my machine, and a GitHub Actions
secret the deploy job uses to decrypt during a rollout.

## Shipping: distroless container on Lightsail, fronted by CloudFront

The deploy target is an AWS Lightsail Container Service — the smallest one
(0.25 vCPU, 512 MB) at about **$7/month**. A multi-stage Dockerfile builds
the release binary with `cargo leptos` on `rust:1.97-bookworm` and copies it
into a distroless image (`gcr.io/distroless/cc-debian12`) that runs as
non-root; the final image contains the binary, the static assets, and
nothing else — no shell, no package manager.

Images are published to a private Amazon ECR repository (scanned on push,
with a lifecycle policy keeping the last ten), which Lightsail pulls through
its ECR image-puller role. CloudFront sits in front — Route53 can't point an
apex domain at a Lightsail container service, so CloudFront (with caching
disabled, since pages are server-rendered) bridges `kenesparta.dev` to the
service and terminates TLS with an ACM certificate. A second CloudFront
distribution, `cdn.kenesparta.dev`, serves images and heavy static assets
from a private S3 bucket (Origin Access Control, 24-hour cache) — including
the images in this post.

All of it — Lightsail, CloudFront, Route53, ACM, ECR, IAM — is Terraform
under `tf/`, with state in S3.

## CI/CD without long-lived keys

Pushing a version tag (`vX.Y.Z`) triggers GitHub Actions:

1. **Build & push**: build the Docker image (with layer caching) and push it
   to the private ECR repo, tagged with the version and `latest`.
2. **Deploy**: assume an AWS role via **OIDC** — no access keys stored in
   GitHub — and run a `terraform apply` scoped (`-target`) to the single
   Lightsail deployment resource, with the tag as the image version.
   Terraform stays the single source of truth for what is running — image
   and environment alike — and a concurrency group guarantees two rollouts
   never race over the shared state.

The same rollout runs locally as `make rollout VERSION=vX.Y.Z` when I don't
want to wait for a runner.

## What's next

The first version of this site ran on App Runner with DynamoDB and
long-lived AWS keys. None of that survived: today it is Lightsail,
PostgreSQL through SQLx, sops, and OIDC — simpler and cheaper at every
layer. Next on the list: RSS, syntax highlighting for code blocks, and more
posts in this series digging into individual pieces (the ingest CLI and the
hydration setup are both worth their own write-ups).

The whole thing is a reminder that "small" doesn't have to mean "static": a
real server, a real database, and real infrastructure can still fit in a
weekend of setup and a hobby budget.
