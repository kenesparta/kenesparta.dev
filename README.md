# kenesparta.dev

Personal portfolio website built with Leptos (Rust full-stack web framework) and deployed on AWS App Runner.

## Tech Stack

- **Frontend/Backend**: [Leptos 0.8.0](https://leptos.dev/) - Full-stack Rust framework with SSR and hydration
- **Web Server**: [Axum 0.8.0](https://github.com/tokio-rs/axum) - Rust web framework
- **Compression**: tower-http with Brotli and Gzip support
- **Styling**: SCSS (compiled via cargo-leptos)
- **Testing**: Playwright for end-to-end tests
- **Infrastructure**: Terraform (AWS App Runner, ECR, Route53, ACM)
- **CI/CD**: GitHub Actions with AWS OIDC authentication

## Architecture

```
Internet
   ↓
Route53 (kenesparta.dev)
   ↓
AWS App Runner (HTTPS, auto-scaling)
   ↓
Leptos App (Axum + Brotli compression)
```

## Prerequisites

### Required Tools

- **Rust nightly**: `rustup toolchain install nightly --allow-downgrade`
- **WASM target**: `rustup target add wasm32-unknown-unknown`
- **cargo-leptos**: `cargo install cargo-leptos --locked`
- **sass**: `npm install -g sass`

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

Manages App Runner service, ECR, Route53, ACM certificates, and DynamoDB.

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
├── site/                   # Main Leptos application
│   ├── src/
│   │   ├── main.rs        # Server entry point (Axum)
│   │   ├── lib.rs         # Library entry point
│   │   ├── app.rs         # Root app component with routing
│   │   ├── components/    # Reusable UI components
│   │   ├── pages/         # Route page components
│   │   └── constants.rs   # Application constants
│   ├── style/             # SCSS stylesheets
│   ├── public/            # Static assets
│   ├── end2end/           # Playwright tests
│   ├── Cargo.toml         # Rust dependencies
│   └── Dockerfile         # Multi-stage build
│
├── tf/                         # Terraform infrastructure
│   ├── app-runner-ke-dev.tf   # App Runner service and custom domain
│   ├── ecr.tf                 # ECR repository
│   ├── iam-*.tf               # IAM roles for GitHub Actions and App Runner
│   ├── dns-*.tf               # Route53 zones and records
│   ├── acm.tf                 # ACM certificate for SSL/TLS
│   └── dynamodb.tf            # DynamoDB table for blog posts
│
├── .github/workflows/     # CI/CD pipelines
└── Makefile              # Build shortcuts
```

## AWS Resources

### Compute (App Runner)
- **Service**: `kenesparta-dev`
- **Resources**: 256 CPU units, 512 MB memory
- **Port**: 3000
- **Health Checks**: HTTP on path `/`
- **Auto-scaling**: Managed by App Runner
- **HTTPS**: Automatic TLS termination

### Container Registry
- **ECR Repository**: `kenesparta-dev`
- **Lifecycle Policy**: Keeps only the latest image
- **Image Scanning**: Enabled on push

### Security
- **IAM Roles**: OIDC federation for GitHub Actions (no long-lived credentials)
- **App Runner Access Role**: For pulling images from ECR
- **Instance Role**: For DynamoDB access
- **SSL/TLS**: ACM certificate for `*.kenesparta.dev`

### DNS
- **Route53**: A record alias to App Runner service
- **Custom Domain**: `kenesparta.dev` with automatic certificate validation

### Database
- **DynamoDB Table**: `kenesparta-blog-posts`
- **Billing**: Pay-per-request (on-demand)
- **Features**: Point-in-time recovery, server-side encryption

## CI/CD Pipeline

GitHub Actions workflow automatically:
1. Builds Docker image on push to `main` or version tags
2. Pushes image to AWS ECR
3. Triggers App Runner deployment

### Required Secrets

- `AWS_ROLE_ARN`: IAM role for GitHub Actions OIDC

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
- Style file: `style/global.scss`

### Feature Flags

- `ssr`: Server-side rendering (Axum, tokio, leptos_axum, tower-http)
- `hydrate`: Client-side hydration (WASM, wasm-bindgen)

### Cost Optimization

- App Runner with minimal resources (256 CPU / 512 MB)
- Pay-per-use model (scales to zero when idle)
- No ALB or NAT Gateway required
- Simplified infrastructure (no VPC management)

## License

This project is personal portfolio code. Feel free to reference the architecture and setup patterns.

## Contributing

This is a personal portfolio project, but feel free to open issues for bugs or suggestions.
