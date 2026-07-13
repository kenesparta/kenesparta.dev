# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a personal portfolio website built with Leptos (Rust full-stack web framework) using Axum as the backend server. The site is deployed using Docker to AWS App Runner with infrastructure managed through Terraform.

**Tech Stack:**
- **Frontend/Backend**: Leptos 0.8.0 (full-stack Rust framework with SSR and hydration)
- **Web Server**: Axum 0.8.0
- **Styling**: SCSS (global.scss)
- **Testing**: Playwright (end-to-end tests)
- **Architecture**: DDD / hexagonal — a Cargo workspace with a library crate per Bounded Context (`crates/bc-*`), a `shared-kernel`, and a single binary (`apps/backend`) that hosts the Leptos app plus all adapters and wiring
- **Containerization**: Docker (multi-stage build)
- **Infrastructure**: Terraform (App Runner, ECR, Route53, DynamoDB) — still present under `tf/`, unchanged
- **CI/CD**: GitHub Actions (`publish-image.yml`) builds the Docker image and publishes it to GHCR (`ghcr.io/kenesparta/kenespartadev`); it does **not** deploy

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
│           ├── domain/        # model.rs (BlogPost/Summary/PostStatus), repository.rs (BlogRepository port), errors.rs
│           └── application/   # use_cases.rs (List/GetBySlug/GetById), dto.rs (BlogPostDTO…)
├── apps/
│   └── backend/           # The Leptos app (SSR bin + hydrate lib) + all adapters
│       ├── src/
│       │   ├── main.rs · lib.rs           # server + wasm entry points
│       │   ├── configuration.rs           # env config
│       │   ├── composition.rs             # DI Container (wires adapters → use cases)
│       │   ├── http.rs                     # ServerState + server-fn handler
│       │   ├── persistence/blog_dynamodb.rs   # DynamoBlogRepository (implements the port)
│       │   └── app/                        # UI: app.rs (routing/shell), components/, pages/, constants.rs, api.rs (server fns)
│       ├── style/            # SCSS stylesheets
│       ├── public/           # Static assets
│       ├── end2end/          # Playwright tests
│       ├── Cargo.toml        # Leptos config (output-name, site-root, …)
│       └── ...
├── Dockerfile             # Multi-stage build (builds from apps/backend)
├── tf/                        # Terraform infrastructure
│   ├── app-runner-ke-dev.tf  # App Runner service and custom domain
│   ├── iam-*.tf              # IAM roles for GitHub Actions and App Runner
│   ├── dns-*.tf              # Route53 zones and records
│   ├── acm.tf                # ACM certificate for SSL/TLS
│   ├── ecr.tf                # ECR repository configuration
│   └── dynamodb.tf           # DynamoDB table for blog posts
├── .github/workflows/    # CI/CD pipelines
└── Makefile              # Build shortcuts
```

## Development Commands

### Local Development

**Prerequisites:**
- Rust nightly toolchain: `rustup toolchain install nightly --allow-downgrade`
- WASM target: `rustup target add wasm32-unknown-unknown`
- cargo-leptos: `cargo install cargo-leptos --locked`
- sass: `npm install -g sass`
- Playwright deps (for tests): `cd site/end2end && npm install`

**Running the development server:**
```bash
cd apps/backend
cargo leptos watch
```
This starts the dev server with hot-reload at http://0.0.0.0:3000

**Building for production:**
```bash
cd apps/backend
cargo leptos build --release
```
Output: `target/release/backend` (binary) and `target/kdevsite/` (site assets)

**Type-checking the workspace (fast, no cargo-leptos):**
```bash
cargo check -p backend --features ssr                                    # server build
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

**Running the container:**
```bash
docker run -p 3000:3000 kenespartadev
```

### Terraform Infrastructure

The `tf/` directory requires a `.env` file with AWS SSO profile configuration:
```bash
TF_VAR_aws_sso_profile=your-profile-name
```

**Infrastructure Management:**
```bash
cd tf
make login       # AWS SSO login
make dev/plan    # Plan changes
make dev/apply   # Apply changes
make dev/destroy # Destroy resources
```

Terraform state is stored remotely in S3 bucket `tf.kenesparta.dev`.

## Architecture Notes

### Leptos Application Structure

The application uses Leptos's full-stack architecture with two compilation targets:
- **Server (SSR)**: Compiled with `ssr` feature, runs on Axum server
- **Client (WASM)**: Compiled with `hydrate` feature, runs in browser

**DDD layering (data flow):**
UI (`app/pages`, `app/components`) → server functions (`app/api.rs`) → use cases (`bc-blog/application`) via the DI `Container` from Leptos context → `BlogRepository` port → `DynamoBlogRepository` adapter (`persistence/`). The Bounded Context (`crates/bc-blog`) has no runtime/HTTP/AWS dependencies; those live only in `apps/backend`. `bc-blog` is compiled for both wasm and SSR so the UI can use its DTOs directly.

**Routing:**
Routes are defined in `apps/backend/src/app.rs` using leptos_router:
- `/` → HomePage
- `/about` → About
- `/blog` → Blog
- `/experience` → Experience
- `/projects` → Projects

The navigation bar (StickyNavBar) is conditionally rendered on all pages except the home page.

**Components:**
- Components are in `apps/backend/src/app/components/` (header, social links, navigation)
- Pages are in `apps/backend/src/app/pages/` (individual route handlers)
- Most pages currently show "coming soon" placeholders

**Styling:**
- Global SCSS is defined in `apps/backend/style/global.scss`
- Leptos config specifies: `style-file = "style/global.scss"`
- Compiled CSS is served at `/pkg/kenespartadev.css`

### Docker Deployment

Multi-stage Dockerfile (build context = workspace root):
1. **Builder stage**: Uses `rust:1.97-bookworm`, installs cargo-leptos, `COPY . .`, runs `cd apps/backend && cargo leptos build --release`
2. **Runtime stage**: Uses distroless image, copies the `backend` binary + `kdevsite/` site assets, runs as non-root

Environment variables for production:
- `LEPTOS_OUTPUT_NAME=kenespartadev`
- `LEPTOS_SITE_ADDR="0.0.0.0:3000"`
- `LEPTOS_SITE_ROOT=/app/kdevsite`
- `LEPTOS_SITE_PKG_DIR=pkg`
- `RUST_LOG="info"`

### AWS Infrastructure (App Runner)

The application is deployed on AWS using App Runner with ECR:

**App Runner Service:**
- Service name: `kenesparta-dev`
- Resources: 256 CPU units, 512 MB memory
- Port: 3000
- Health checks: HTTP on path `/`
- Auto-scaling: Managed by App Runner
- HTTPS: Automatic TLS termination
- Custom domain: `kenesparta.dev` with automatic certificate validation

**ECR (Elastic Container Registry):**
- Repository: `kenesparta-dev`
- Lifecycle policy: Keeps only the latest image
- Image scanning enabled on push

**IAM Roles:**
- GitHub Actions OIDC role: For pushing images to ECR and triggering deployments
- App Runner access role: For pulling images from ECR
- App Runner instance role: For DynamoDB access at runtime

**DNS:**
- `kenesparta.dev` A record points to App Runner service using Route53 alias
- SSL/TLS certificate via ACM with automatic DNS validation

**DynamoDB:**
- Table: `kenesparta-blog-posts`
- Billing: Pay-per-request (on-demand)
- Global secondary index on status + created_at
- Point-in-time recovery enabled
- Server-side encryption enabled

**Cost Optimization:**
- App Runner with minimal resources (256 CPU / 512 MB)
- Pay-per-use model (scales to zero when idle)
- No ALB, NAT Gateway, or VPC required
- Simplified infrastructure reduces operational overhead

### CI/CD Pipeline

GitHub Actions workflow (`.github/workflows/publish-image.yml`):
- Triggers on push to `main` or version tags (`v*.*.*`) (ignores `**.md` and `tf/**`)
- Builds the Docker image and pushes it to GHCR: `ghcr.io/kenesparta/kenespartadev:latest` and `:<short sha>`
- Uses the built-in `GITHUB_TOKEN` (`packages: write`) — no secrets to configure
- Does **not** deploy. The `tf/` App Runner + ECR infrastructure is untouched; repoint the deploy target at the GHCR image when ready

**First push:** the GHCR package is created private — make it public (Package settings → Change visibility) if the deploy target must pull it without credentials.

## Cargo.toml Configuration

Leptos package metadata lives in `apps/backend/Cargo.toml`:
- `output-name = "kenespartadev"`
- `site-root = "target/kdevsite"`
- `site-addr = "0.0.0.0:3000"`
- `reload-port = 3001` (for hot-reload)
- `end2end-cmd = "npx playwright test"`

## Testing

Playwright tests are located in `site/end2end/tests/`.

Test configuration in `site/end2end/playwright.config.ts`:
- Runs tests in parallel (chromium, firefox, webkit)
- 30s timeout per test
- HTML reporter
