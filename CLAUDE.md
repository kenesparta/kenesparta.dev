# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a personal portfolio website built with Leptos (Rust full-stack web framework) using Axum as the backend server. The site is deployed using Docker to AWS ECS with infrastructure managed through Terraform.

**Tech Stack:**
- **Frontend/Backend**: Leptos 0.8.0 (full-stack Rust framework with SSR and hydration)
- **Web Server**: Axum 0.8.0
- **Styling**: SCSS (global.scss)
- **Testing**: Playwright (end-to-end tests)
- **Containerization**: Docker (multi-stage build)
- **Infrastructure**: Terraform (DNS via Route53, backend S3 state)
- **CI/CD**: GitHub Actions (deploys to AWS ECR/ECS)

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
├── tf/                   # Terraform infrastructure
│   ├── dns/              # DNS, ECS/ECR, CDN infrastructure
│   │   ├── vpc.tf        # VPC, subnets, internet gateway, route tables
│   │   ├── ecs.tf        # ECS cluster, ECR repo, security groups
│   │   ├── iam-*.tf      # IAM roles for GitHub Actions and ECS
│   │   ├── cdn.tf        # CloudFront and S3 for static CDN
│   │   └── dns-*.tf      # Route53 zones and records
│   └── backend/          # Terraform backend infrastructure
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

Both `tf/dns/` and `tf/backend/` require a `.env` file with AWS SSO profile configuration:
```bash
TF_VAR_aws_sso_profile=your-profile-name
```

**DNS Infrastructure (tf/dns/):**
```bash
cd tf/dns
make login       # AWS SSO login
make dev/plan    # Plan changes
make dev/apply   # Apply changes
make dev/destroy # Destroy resources
```

**Backend Infrastructure (tf/backend/):**
```bash
cd tf/backend
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

### AWS Infrastructure (ECS/ECR)

The application is deployed on AWS using ECS Fargate with CloudFront CDN (no ALB to save costs):

**VPC (Virtual Private Cloud):**
- Custom VPC with CIDR 10.0.0.0/16
- 3 public subnets across 3 availability zones (10.0.1.0/24, 10.0.2.0/24, 10.0.3.0/24)
- Internet Gateway for public internet access
- Route tables configured for internet routing
- VPC Flow Logs enabled (7-day retention for debugging)
- No NAT Gateway (saves ~$32/month, not needed for public ECS tasks)

**ECR (Elastic Container Registry):**
- Repository: `kenesparta-dev`
- Lifecycle policy: Keeps only last 10 images
- Image scanning enabled on push

**ECS (Elastic Container Service):**
- Cluster: `kenesparta-cluster`
- Service: `kenesparta-service`
- Task Definition: `kenesparta-dev`
- Container: `kenesparta-app`
- Launch Type: Fargate (serverless)
- Resources: 256 CPU units, 512 MB memory
- Networking: awsvpc mode with public IP assignment
- Single task (cost-optimized for personal site)

**Service Discovery (AWS Cloud Map):**
- Public DNS namespace: `ecs.kenesparta.dev`
- Service DNS: `app.ecs.kenesparta.dev`
- Automatically updates DNS when ECS task IP changes
- TTL: 10 seconds for fast failover
- Routing: MULTIVALUE for multiple task IPs

**CloudFront CDN:**
- Distribution with custom domain `kenesparta.dev`
- Origin: Service Discovery DNS (`app.ecs.kenesparta.dev:3000`)
- HTTPS termination with ACM certificate for `*.kenesparta.dev`
- Automatic HTTP to HTTPS redirect
- Caching with custom policies for dynamic content
- Compression enabled (Brotli + Gzip)
- Price class: North America & Europe only (PriceClass_100)

**Security:**
- ECS Tasks Security Group: Allows inbound 3000 from internet (CloudFront has no fixed IPs)
- IAM roles using OIDC federation for GitHub Actions (no long-lived credentials)
- Task execution role for pulling images and writing logs
- Task role for runtime permissions

**DNS:**
- `kenesparta.dev` A record points to CloudFront using Route53 alias
- `app.ecs.kenesparta.dev` managed by Service Discovery (points to ECS task IPs)
- SSL/TLS certificate via ACM with automatic DNS validation

**Logging:**
- CloudWatch log group: `/ecs/kenesparta-dev`
- Log retention: 7 days
- Container Insights enabled for monitoring

**Cost Optimization:**
- No Application Load Balancer (~$16-20/month saved)
- No NAT Gateway (~$32/month saved, using public subnets only)
- Single Fargate task with minimal resources (256 CPU / 512 MB)
- CloudFront is pay-per-use (minimal cost for low traffic)
- Service Discovery is low cost (~$1/month per namespace)
- VPC Flow Logs for debugging only (can be disabled in production)

### CI/CD Pipeline

GitHub Actions workflow (`.github/workflows/page.yml`):
- Triggers on push to `main` or version tags (`v*.*.*`)
- Builds Docker image and pushes to AWS ECR
- Updates ECS task definition and deploys to ECS cluster

**GitHub Secrets Required:**
- `AWS_ROLE_ARN`: IAM role ARN from `tf/dns/iam-resume-s3.tf` output

**Workflow Configuration:**
- ECR Repository: `kenesparta-dev`
- ECS Cluster: `kenesparta-cluster`
- ECS Service: `kenesparta-service`
- ECS Task Definition: `kenesparta-dev`
- Container Name: `kenesparta-app`

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
