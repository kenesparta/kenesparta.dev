+++
title = "Building kenesparta.dev: a Rust site from browser to database"
summary = "How this site is built — Leptos SSR with hydration, a hexagonal Cargo workspace, Markdown posts ingested into Postgres, and a $7/month deploy on AWS Lightsail behind CloudFront."
date = "2026-07-21T00:00:00Z"
tags = ["rust", "leptos", "aws", "terraform", "postgres"]
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

The site is built with [Leptos](https://leptos.dev), a full-stack Rust web
framework, served by [Axum](https://github.com/tokio-rs/axum). The same
codebase compiles twice:

- **Server (SSR)**: a native binary that renders every page to HTML, so the
  first paint needs no JavaScript at all.
- **Client (WASM)**: the same components compiled to WebAssembly, which
  *hydrate* the server-rendered HTML and take over interactivity.

There is no REST API layer to keep in sync. Leptos *server functions* let a
component call what looks like a plain async Rust function; on the server it
runs directly, and in the browser it becomes a typed RPC call. One function
signature, checked by the compiler on both sides.

Styling is deliberately boring: plain CSS files under `style/parts/`,
concatenated into a single bundle and minified. No Tailwind, no Sass, no
build-time surprises.

## A hexagonal workspace

The repository is a Cargo workspace laid out along DDD lines:

- `crates/bc-blog` — the *blog* Bounded Context: domain model (`BlogPost`,
  `PostStatus`), use cases, and a `BlogRepository` port. It has **no** HTTP,
  SQL, or runtime dependencies, so it compiles unchanged for both the server
  and the WASM client.
- `crates/shared-kernel` — small cross-cutting types (IDs, datetimes, domain
  errors).
- `apps/backend` — the only binary. It hosts the Leptos app and every adapter:
  the Axum server, the SQLx implementation of `BlogRepository`, configuration,
  and the dependency-injection wiring.

The flow for a page like `/blog/{slug}` is:

```
UI component → server function → use case (bc-blog) → repository port
                                                        └─ Postgres adapter (SQLx)
```

Is this over-engineered for a personal site? Absolutely — and that's the
point. The site is a small, low-stakes place to practice the architecture I
want to be fluent in, with the compiler enforcing the boundaries.

## Posts are Markdown, the database is a cache of the repo

Every post is a Markdown file with TOML frontmatter under `content/posts/`,
version-controlled with the rest of the site. Publishing is one command:

```sh
make blog/publish
```

An ingest CLI renders the Markdown with `pulldown-cmark` and *upserts* each
post into PostgreSQL, keyed by slug. Re-running it is idempotent: `post_id`
and `created_at` are preserved, so URLs never break and edits are just
re-ingests. Posts default to `draft` status — nothing goes public by
omission.

The database is a Lightsail managed Postgres instance. The schema is owned by
SQLx migrations embedded in the binary and applied at startup, so a fresh
database bootstraps itself.

<!-- TODO image: screenshot of `make blog/publish` output
     https://cdn.kenesparta.dev/blog/building-kenesparta-dev/publish.webp -->

## Secrets live in the repo — encrypted

Database credentials sit in `secrets/{dev,prod}.enc.env`, committed to the
repository but encrypted with [sops](https://github.com/getsops/sops) and
[age](https://age-encryption.org). Local commands run under
`sops exec-env`, and Terraform decrypts the production file at apply time to
inject `DATABASE_URL` into the container. The private key never leaves my
machine, and CI never needs it.

## Shipping: distroless container on Lightsail, fronted by CloudFront

The deploy target is an AWS Lightsail Container Service — the smallest one
(0.25 vCPU, 512 MB) at about **$7/month**. A multi-stage Dockerfile builds
the release binary with `cargo leptos` and copies it into a distroless image
that runs as non-root; the final image contains the binary, the static
assets, and nothing else.

The image is published to a private Amazon ECR repository, which Lightsail
pulls through its ECR image-puller role. CloudFront sits in front — Route53
can't point an apex domain at a Lightsail container service, so CloudFront
(with caching disabled, since pages are server-rendered) bridges
`kenesparta.dev` to the service and terminates TLS with an ACM certificate.

All of it — Lightsail, CloudFront, Route53, ACM, IAM — is Terraform under
`tf/`, with state in S3.

## CI/CD without long-lived keys

Pushing a version tag triggers GitHub Actions:

1. Build the Docker image and push it to the private ECR repo, tagged with
   the version and `latest`.
2. Assume an AWS role via **OIDC** — no access keys stored in GitHub — and
   run a `terraform apply` scoped to the Lightsail deployment resource, so
   Terraform stays the single source of truth for what is running (image
   version and environment alike).

The container's environment comes from the same sops-encrypted file as
everything else, decrypted in the runner at apply time — one secret, one
pipeline, no drift between what Terraform knows and what production runs.

## What's next

The site started life on App Runner with DynamoDB and static AWS keys; the
current shape — Lightsail, Postgres, sops, OIDC — is simpler and cheaper at
every layer. Next on the list: RSS, syntax highlighting for code blocks, and
more posts in this series digging into individual pieces (the ingest CLI and
the hydration setup are both worth their own write-ups).

The whole thing is a reminder that "small" doesn't have to mean "static": a
real server, a real database, and real infrastructure can still fit in a
weekend of setup and a hobby budget.
