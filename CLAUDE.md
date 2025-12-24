# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a personal portfolio website built with Leptos (Rust full-stack web framework) using Axum as the backend server. The site is deployed using Docker to AWS App Runner with infrastructure managed through Terraform.

**Tech Stack:**
- **Frontend/Backend**: Leptos 0.8.0 (full-stack Rust framework with SSR and hydration)
- **Web Server**: Axum 0.8.0
- **Styling**: SCSS (global.scss)
- **Testing**: Playwright (end-to-end tests)
- **Containerization**: Docker (multi-stage build)
- **Infrastructure**: Terraform (App Runner, ECR, Route53, DynamoDB)
- **CI/CD**: GitHub Actions (deploys to AWS ECR/App Runner)

## Repository Structure

```
.
├── site/                  # Main Leptos application
│   ├── src/
│   │   ├── app.rs        # Root app component with routing
│   │   ├── main.rs       # Server entry point
│   │   ├── lib.rs        # Library entry point
│   │   ├── components/   # Reusable UI components
│   │   ├── pages/        # Route page components
│   │   └── constants.rs  # Application constants
│   ├── style/            # SCSS stylesheets
│   ├── public/           # Static assets
│   ├── end2end/          # Playwright tests
│   ├── Cargo.toml        # Rust dependencies and Leptos config
│   └── Dockerfile        # Multi-stage Docker build
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
cd site
cargo leptos watch
```
This starts the dev server with hot-reload at http://0.0.0.0:3000

**Building for production:**
```bash
cd site
cargo leptos build --release
```
Output: `target/release/kenespartadev` (binary) and `target/kdevsite/` (site assets)

**Running end-to-end tests:**
```bash
cd site
cargo leptos end-to-end          # Debug mode
cargo leptos end-to-end --release # Release mode
```

### Docker

**Building the Docker image:**
```bash
cd site
docker build -t kenespartadev .
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

**Routing:**
Routes are defined in `site/src/app.rs` using leptos_router:
- `/` → HomePage
- `/about` → About
- `/blog` → Blog
- `/experience` → Experience
- `/projects` → Projects

The navigation bar (StickyNavBar) is conditionally rendered on all pages except the home page.

**Components:**
- Components are in `site/src/components/` (header, social links, navigation)
- Pages are in `site/src/pages/` (individual route handlers)
- Most pages currently show "coming soon" placeholders

**Styling:**
- Global SCSS is defined in `site/style/global.scss`
- Leptos config specifies: `style-file = "style/global.scss"`
- Compiled CSS is served at `/pkg/kenespartadev.css`

### Docker Deployment

Multi-stage Dockerfile:
1. **Builder stage**: Uses `rust:1.90`, installs cargo-leptos, builds release binary
2. **Runtime stage**: Uses distroless image, copies binary + site assets, runs as non-root

Environment variables for production:
- `LEPTOS_SITE_ADDR="0.0.0.0:3000"`
- `LEPTOS_SITE_ROOT=./kdevsite`
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

GitHub Actions workflow (`.github/workflows/page.yml`):
- Triggers on push to `main` or version tags (`v*.*.*`)
- Builds Docker image and pushes to AWS ECR
- Triggers App Runner deployment

**GitHub Secrets Required:**
- `AWS_ROLE_ARN`: IAM role ARN from `tf/iam-main.tf` output

**Workflow Configuration:**
- ECR Repository: `kenesparta-dev`
- App Runner Service: `kenesparta-dev`

## Cargo.toml Configuration

Package metadata includes Leptos-specific configuration:
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
