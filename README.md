# kenesparta.dev

Personal portfolio website built with Leptos (Rust full-stack web framework), deployed as a container on AWS Lightsail behind CloudFront.

## Tech Stack

- **Frontend/Backend**: [Leptos 0.8.0](https://leptos.dev/) - Full-stack Rust framework with SSR and hydration
- **Web Server**: [Axum 0.8.0](https://github.com/tokio-rs/axum) - Rust web framework
- **Compression**: tower-http with Brotli and Gzip support
- **Styling**: plain CSS — `style/parts/*.css` concatenated into `style/main.css` by `make css` (no Sass), minified by cargo-leptos via lightningcss
- **Testing**: Playwright for end-to-end tests
- **Database**: PostgreSQL (Lightsail managed database) via SQLx — blog posts authored as Markdown in `content/posts/`
- **Secrets**: [sops](https://github.com/getsops/sops) + [age](https://github.com/FiloSottile/age) — encrypted dotenv files committed under `secrets/`
- **Infrastructure**: Terraform (AWS Lightsail Containers, CloudFront, Route53, ACM)
- **CI/CD**: GitHub Actions — publishes the image to a private ECR repo, then rolls it out on Lightsail with `terraform apply -target` (AWS OIDC)

## Architecture

```
Internet
   ↓
Route53 (kenesparta.dev apex ALIAS)
   ↓
CloudFront (HTTPS, ACM cert)
   ↓
Lightsail Container Service  ←  image: private ECR repo `kenespartadev`
   ↓
Leptos App (Axum + Brotli)   →  Lightsail PostgreSQL (blog)
```

## Prerequisites

### Required Tools

- **Rust nightly**: `rustup toolchain install nightly --allow-downgrade`
- **WASM target**: `rustup target add wasm32-unknown-unknown`
- **cargo-leptos**: `cargo install cargo-leptos --locked`
- **sops + age** (secrets): `brew install sops age`

### For Infrastructure Management

- **Terraform**: v1.5+
- **AWS CLI**: v2+ with SSO configured
- **Docker**: For local container testing

### For Testing

- **Node.js**: v18+ (for Playwright)
- **Playwright**: `cd apps/backend/end2end && npm install`

## Getting Started

### Local Development

The easy path (app + PostgreSQL, hot-reload):

```bash
make dev/up
```

Or on the host (needs the dev database running: `docker compose -f docker-compose.dev.yml up -d postgres`):

```bash
sops exec-env secrets/dev.enc.env 'sh -c "cd apps/backend && cargo leptos watch"'
```

This starts the development server with hot-reload at http://localhost:3000.
The app requires `DATABASE_URL` and fails fast without it — `make dev/up` sets
it, and `sops exec-env` injects it from the encrypted dev secrets.

### Building for Production

```bash
cd apps/backend
cargo leptos build --release
```

Output:
- Binary: `target/release/backend`
- Site assets: `target/kdevsite/`

### Running Tests

```bash
cd apps/backend

# Debug mode
cargo leptos end-to-end

# Release mode
cargo leptos end-to-end --release
```

## Secrets (sops + age)

Secrets live encrypted in the repo (`secrets/dev.enc.env`, `secrets/prod.enc.env`,
dotenv format) and are safe to commit. Encryption uses [sops](https://github.com/getsops/sops)
with an [age](https://github.com/FiloSottile/age) key that stays **outside** the repo.

One-time setup on a new machine:

```bash
brew install sops age
mkdir -p ~/.config/sops/age && age-keygen -o ~/.config/sops/age/keys.txt
# add the printed "public key: age1..." to .sops.yaml and run `make secrets-rotate`
```

> **macOS note**: sops' default age key path on macOS is
> `~/Library/Application Support/sops/age/keys.txt`. The Makefiles export
> `SOPS_AGE_KEY_FILE=~/.config/sops/age/keys.txt`; add the same export to your
> shell profile for bare `sops` commands.

Day-to-day:

```bash
make secrets          # edit dev secrets in $EDITOR (re-encrypts on save)
make secrets-prod     # edit prod secrets
make secrets-view ENV=prod   # print decrypted secrets
make secrets-rotate   # re-encrypt after changing recipients in .sops.yaml
```

The only secret today is `DATABASE_URL`. Terraform decrypts `secrets/prod.enc.env`
(carlpett/sops provider) and injects it into the Lightsail deployment; CI never
sees the age key.

## Writing Blog Posts

Posts are Markdown files under `content/posts/` with TOML frontmatter between
`+++` fences:

```markdown
+++
title = "My Post"
summary = "One-line summary shown on the blog card."
date = "2026-07-21T00:00:00Z"        # RFC 3339; becomes created_at/published_at
tags = ["rust"]                      # optional, default []
status = "published"                 # optional, default "draft" (not public)
# author = "Ken Esparta"             # optional, this is the default
# slug = "my-post"                   # optional, defaults to the file stem
+++

Markdown body (tables, footnotes and strikethrough enabled)...
```

Publish with the ingest CLI — it renders Markdown → HTML (pulldown-cmark) and
upserts each post **by slug** (idempotent; `post_id` and `created_at` are
preserved, so URLs never break):

```bash
make blog/ingest             # into the dev database
make blog/publish            # into PRODUCTION
```

Note: with the Lightsail database's public mode off, your machine cannot reach
it — temporarily enable it around a publish:

```bash
aws lightsail update-relational-database --relational-database-name <db> --publicly-accessible
make blog/publish
aws lightsail update-relational-database --relational-database-name <db> --no-publicly-accessible
```

## Docker

### Build Image

```bash
docker build -t kenespartadev .   # build context is the workspace root
```

### Run Container

```bash
docker run -p 3000:3000 kenespartadev
```

The multi-stage Dockerfile:
1. **Builder**: Uses `rust:1.97`, installs cargo-leptos, builds release binary
2. **Runtime**: Uses distroless image, runs as non-root user

## Infrastructure

### Setup

The `tf/` directory requires a `.env` file with AWS SSO profile:

```bash
TF_VAR_aws_sso_profile=your-profile-name
```

Terraform also needs the age key (`SOPS_AGE_KEY_FILE`, exported by `tf/Makefile`)
to decrypt `secrets/prod.enc.env` at plan/apply time.

### Infrastructure Management

Manages the Lightsail Container Service, CloudFront, Route53, and ACM certificates.
The PostgreSQL database is a Lightsail managed database created outside Terraform;
its connection string lives in `secrets/prod.enc.env`.

```bash
cd tf
make login       # AWS SSO login
make dev/plan    # Plan changes
make dev/apply   # Apply changes
make dev/destroy # Destroy resources
```

**Note**: Terraform state is stored in S3 bucket `tf.kenesparta.dev`

## Project Structure

```
.
├── content/
│   └── posts/              # Blog posts: Markdown + TOML frontmatter (source of truth)
├── crates/
│   ├── shared-kernel/      # Cross-cutting types (DomainError, Datetime, PostUuid)
│   └── bc-blog/            # Blog Bounded Context (domain + application, no runtime/IO)
├── apps/
│   └── backend/           # Leptos app (SSR + hydrate) + all adapters
│       ├── src/           # main.rs, lib.rs, app/ (UI), persistence/, composition.rs, http.rs
│       ├── src/bin/       # ingest.rs (markdown → Postgres CLI, feature `ingest`)
│       ├── migrations/    # SQLx migrations (embedded in the binary)
│       ├── style/         # parts/*.css (source) → main.css (via make css)
│       ├── public/        # Static assets
│       └── end2end/       # Playwright tests
├── secrets/               # sops/age-encrypted dotenv files (safe to commit)
├── .sops.yaml             # sops creation rules (age recipients)
├── Dockerfile             # Multi-stage build (builds from apps/backend)
├── tf/                        # Terraform infrastructure
│   ├── lightsail.tf          # Lightsail Container Service (pulls private ECR) + sops secrets
│   ├── ecr.tf                # Private ECR repo + lifecycle + Lightsail pull policy
│   ├── cloudfront.tf         # CloudFront (fronts the apex) + apex ALIAS record
│   ├── iam-main.tf           # GitHub Actions OIDC role (ECR push + TF state + Lightsail deploy)
│   ├── dns-*.tf              # Route53 zones and records
│   └── acm.tf                # ACM certificate (used by CloudFront)
├── .github/workflows/     # CI/CD pipelines
└── Makefile               # Build shortcuts
```

## AWS Resources

### Compute (Lightsail + CloudFront)
- **Lightsail Container Service**: `kenesparta-app`, power `nano` (0.25 vCPU / 512 MB, ~$7/mo)
- **Image**: pulls the private ECR image through the service's ECR image-puller role (`private_registry_access`)
- **Health Checks**: HTTP on path `/`
- **CloudFront**: fronts the apex (Route53 can't ALIAS to Lightsail); HTTPS via ACM; pure pass-through

### Container Registry
- **ECR**: private repo `kenespartadev` — tags `vX.Y.Z` + `latest`, scan-on-push, lifecycle keeps the last 10 images

### Security
- **IAM Roles**: OIDC federation for GitHub Actions (no long-lived credentials)
- **Lightsail deploy**: the OIDC role runs a `terraform apply` scoped to the deployment resource after pushing to ECR
- **Secrets**: sops/age-encrypted in the repo; decrypted by Terraform locally (`SOPS_AGE_KEY_FILE`) and in CI (`SOPS_AGE_KEY` repo secret)
- **SSL/TLS**: ACM certificate for `kenesparta.dev` + `*.kenesparta.dev` (used by CloudFront); `DATABASE_URL` uses `sslmode=require`

### DNS
- **Route53**: apex ALIAS `kenesparta.dev` → CloudFront
- **Custom Domain**: certificate validated via DNS

### Database
- **Lightsail Managed PostgreSQL**: created manually in the Lightsail console (outside Terraform)
- **Access**: the container connects with `DATABASE_URL` (TLS, `sslmode=require`); with public mode off, only Lightsail resources in the region can reach it
- **Schema**: owned by SQLx migrations in `apps/backend/migrations/`, run automatically at app startup and by the ingest CLI

## CI/CD Pipeline

GitHub Actions (`publish-image.yml`) on version tags (`vX.Y.Z`):
1. **`build-push`**: builds the Docker image and pushes it to the private ECR repo (`:vX.Y.Z` + `:latest`)
2. **`deploy`**: rolls it out on Lightsail with `terraform apply -auto-approve -target` of the deployment resource (`TF_VAR_image_version=<tag>`); the sops provider decrypts `secrets/prod.enc.env` in the runner

The same rollout can be run locally: `cd tf && make rollout VERSION=vX.Y.Z`.

### Required Secrets

- `AWS_ROLE_ARN`: OIDC role ARN for GitHub Actions (both jobs)
- `SOPS_AGE_KEY`: the age private key (`AGE-SECRET-KEY-…` line from `~/.config/sops/age/keys.txt`), used by the deploy job to decrypt `secrets/prod.enc.env`

## Routes

The application uses leptos_router with the following routes:

- `/` - Home page
- `/about` - About page
- `/blog` - Blog (list of published posts)
- `/blog/:slug` - Blog post detail
- `/experience` - Experience timeline (coming soon)
- `/projects` - Projects showcase (coming soon)

Navigation bar is conditionally rendered on all pages except home.

## Compression

Application-level compression via tower-http:
- **Brotli**: Primary compression (best ratio)
- **Gzip**: Fallback for older browsers
- **Automatic**: Detects client capabilities

Compression applies to:
- HTML, CSS, JavaScript
- JSON, XML responses
- Text files, SVG images

## Development Notes

### Leptos Configuration

From `Cargo.toml`:
- Output name: `kenespartadev`
- Site root: `target/kdevsite`
- Site address: `0.0.0.0:3000`
- Reload port: 3001 (hot-reload)
- Style file: `style/main.css`

### Feature Flags

- `ssr`: Server-side rendering (Axum, tokio, leptos_axum, tower-http, SQLx)
- `hydrate`: Client-side hydration (WASM, wasm-bindgen)
- `ingest`: the `ingest` bin (`ssr` + pulldown-cmark + toml); excluded from the server build via `required-features`

### Cost Optimization

- Lightsail Container Service `nano` (~$7/mo fixed) + CloudFront pay-per-use
- Private ECR repo with a lifecycle policy (last 10 images, ~$0.10/GB/mo)
- No ALB, NAT Gateway, or VPC required
- Simplified infrastructure, minimal operational overhead

## License

This project is personal portfolio code. Feel free to reference the architecture and setup patterns.

## Contributing

This is a personal portfolio project, but feel free to open issues for bugs or suggestions.
