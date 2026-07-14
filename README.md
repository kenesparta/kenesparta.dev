# kenesparta.dev

Personal portfolio website built with Leptos (Rust full-stack web framework), deployed as a container on AWS Lightsail behind CloudFront.

## Tech Stack

- **Frontend/Backend**: [Leptos 0.8.0](https://leptos.dev/) - Full-stack Rust framework with SSR and hydration
- **Web Server**: [Axum 0.8.0](https://github.com/tokio-rs/axum) - Rust web framework
- **Compression**: tower-http with Brotli and Gzip support
- **Styling**: plain CSS — `style/parts/*.css` concatenated into `style/main.css` by `make css` (no Sass), minified by cargo-leptos via lightningcss
- **Testing**: Playwright for end-to-end tests
- **Infrastructure**: Terraform (AWS Lightsail Containers, CloudFront, Route53, ACM, DynamoDB)
- **CI/CD**: GitHub Actions — publishes the image to GHCR, then triggers a Lightsail redeploy (AWS OIDC)

## Architecture

```
Internet
   ↓
Route53 (kenesparta.dev apex ALIAS)
   ↓
CloudFront (HTTPS, ACM cert)
   ↓
Lightsail Container Service  ←  image: ghcr.io/kenesparta/kenespartadev
   ↓
Leptos App (Axum + Brotli)   →  DynamoDB (blog)
```

## Prerequisites

### Required Tools

- **Rust nightly**: `rustup toolchain install nightly --allow-downgrade`
- **WASM target**: `rustup target add wasm32-unknown-unknown`
- **cargo-leptos**: `cargo install cargo-leptos --locked`

### For Infrastructure Management

- **Terraform**: v1.5+
- **AWS CLI**: v2+ with SSO configured
- **Docker**: For local container testing

### For Testing

- **Node.js**: v18+ (for Playwright)
- **Playwright**: `cd site/end2end && npm install`

## Getting Started

### Local Development

```bash
cd site
cargo leptos watch
```

This starts the development server with hot-reload at http://localhost:3000

### Building for Production

```bash
cd site
cargo leptos build --release
```

Output:
- Binary: `target/release/kenespartadev`
- Site assets: `target/kdevsite/`

### Running Tests

```bash
cd site

# Debug mode
cargo leptos end-to-end

# Release mode
cargo leptos end-to-end --release
```

## Docker

### Build Image

```bash
cd site
docker build -t kenespartadev .
```

### Run Container

```bash
docker run -p 3000:3000 kenespartadev
```

The multi-stage Dockerfile:
1. **Builder**: Uses `rust:1.90`, installs cargo-leptos, builds release binary
2. **Runtime**: Uses distroless image, runs as non-root user

## Infrastructure

### Setup

The `tf/` directory requires a `.env` file with AWS SSO profile:

```bash
TF_VAR_aws_sso_profile=your-profile-name
```

### Infrastructure Management

Manages the Lightsail Container Service, CloudFront, Route53, ACM certificates, and DynamoDB.

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
├── crates/
│   ├── shared-kernel/      # Cross-cutting types (DomainError, Datetime, PostUuid)
│   └── bc-blog/            # Blog Bounded Context (domain + application, no runtime/IO)
├── apps/
│   └── backend/           # Leptos app (SSR + hydrate) + all adapters
│       ├── src/           # main.rs, lib.rs, app/ (UI), persistence/, composition.rs, http.rs
│       ├── style/         # parts/*.css (source) → main.css (via make css)
│       ├── public/        # Static assets
│       └── end2end/       # Playwright tests
├── Dockerfile             # Multi-stage build (builds from apps/backend)
├── tf/                        # Terraform infrastructure
│   ├── lightsail.tf          # Lightsail Container Service (pulls GHCR) + DynamoDB IAM user
│   ├── cloudfront.tf         # CloudFront (fronts the apex) + apex ALIAS record
│   ├── iam-main.tf           # GitHub Actions OIDC role (GHCR build + Lightsail deploy)
│   ├── dns-*.tf              # Route53 zones and records
│   ├── acm.tf                # ACM certificate (used by CloudFront)
│   └── dynamodb.tf           # DynamoDB table for blog posts
├── .github/workflows/     # CI/CD pipelines
└── Makefile               # Build shortcuts
```

## AWS Resources

### Compute (Lightsail + CloudFront)
- **Lightsail Container Service**: `kenesparta-app`, power `nano` (0.25 vCPU / 512 MB, ~$7/mo)
- **Image**: pulls `ghcr.io/kenesparta/kenespartadev:latest` (public) directly from GHCR
- **Health Checks**: HTTP on path `/`
- **CloudFront**: fronts the apex (Route53 can't ALIAS to Lightsail); HTTPS via ACM; pure pass-through

### Container Registry
- **GHCR**: `ghcr.io/kenesparta/kenespartadev` (public). No ECR.

### Security
- **IAM Roles**: OIDC federation for GitHub Actions (no long-lived credentials)
- **Lightsail deploy**: the OIDC role triggers a Lightsail redeploy after the image is on GHCR
- **DynamoDB access**: dedicated IAM user (`kenesparta-lightsail-app`) — Lightsail containers get no IAM role
- **SSL/TLS**: ACM certificate for `kenesparta.dev` + `*.kenesparta.dev` (used by CloudFront)

### DNS
- **Route53**: apex ALIAS `kenesparta.dev` → CloudFront
- **Custom Domain**: certificate validated via DNS

### Database
- **DynamoDB Table**: `kenesparta-blog-posts`
- **Billing**: Pay-per-request (on-demand)
- **Features**: Point-in-time recovery, server-side encryption

## CI/CD Pipeline

GitHub Actions (`publish-image.yml`) on version tags (`v*.*.*`) or manual dispatch:
1. Builds the Docker image and pushes it to GHCR (`:latest` + `:<short sha>`)
2. Triggers a Lightsail redeploy so the container re-pulls the new `:latest`

### Required Secrets

- `AWS_ROLE_ARN`: OIDC role ARN for GitHub Actions (used by the Lightsail deploy job)

## Routes

The application uses leptos_router with the following routes:

- `/` - Home page
- `/about` - About page
- `/blog` - Blog (coming soon)
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

- `ssr`: Server-side rendering (Axum, tokio, leptos_axum, tower-http)
- `hydrate`: Client-side hydration (WASM, wasm-bindgen)

### Cost Optimization

- Lightsail Container Service `nano` (~$7/mo fixed) + CloudFront pay-per-use
- Image on GHCR (free public registry) — no ECR
- No ALB, NAT Gateway, or VPC required
- Simplified infrastructure, minimal operational overhead

## License

This project is personal portfolio code. Feel free to reference the architecture and setup patterns.

## Contributing

This is a personal portfolio project, but feel free to open issues for bugs or suggestions.
