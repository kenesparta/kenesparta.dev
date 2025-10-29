# kenesparta.dev

Personal portfolio website built with Leptos (Rust full-stack web framework) and deployed on AWS ECS.

## Tech Stack

- **Frontend/Backend**: [Leptos 0.8.0](https://leptos.dev/) - Full-stack Rust framework with SSR and hydration
- **Web Server**: [Axum 0.8.0](https://github.com/tokio-rs/axum) - Rust web framework
- **Compression**: tower-http with Brotli and Gzip support
- **Styling**: SCSS (compiled via cargo-leptos)
- **Testing**: Playwright for end-to-end tests
- **Infrastructure**: Terraform (AWS ECS, ALB, Route53, ACM)
- **CI/CD**: GitHub Actions with AWS OIDC authentication

## Architecture

```
Internet
   ↓
Route53 (kenesparta.dev)
   ↓
Application Load Balancer (HTTPS)
   ↓
ECS Fargate Tasks (3 AZs)
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

Both `tf/dns/` and `tf/backend/` require a `.env` file with AWS SSO profile:

```bash
TF_VAR_aws_sso_profile=your-profile-name
```

### DNS Infrastructure (tf/dns/)

Manages ECS cluster, ALB, Route53, ACM certificates, and networking.

```bash
cd tf/dns
make login       # AWS SSO login
make dev/plan    # Plan changes
make dev/apply   # Apply changes
make dev/destroy # Destroy resources
```

### Backend Infrastructure (tf/backend/)

Manages S3 bucket for Terraform state storage.

```bash
cd tf/backend
make login       # AWS SSO login
make dev/plan    # Plan changes
make dev/apply   # Apply changes
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
├── tf/                    # Terraform infrastructure
│   ├── dns/               # Main infrastructure
│   │   ├── vpc.tf        # VPC, subnets, IGW
│   │   ├── ecs.tf        # ECS, ALB, security groups
│   │   ├── iam-*.tf      # IAM roles
│   │   └── dns-*.tf      # Route53, ACM
│   └── backend/           # State management
│
├── .github/workflows/     # CI/CD pipelines
└── Makefile              # Build shortcuts
```

## AWS Resources

### Networking
- **VPC**: Custom VPC (10.0.0.0/16) with 3 public subnets across 3 AZs
- **Internet Gateway**: For public internet access
- **No NAT Gateway**: Cost optimization (using public subnets)

### Compute
- **ECS Cluster**: `kenesparta-cluster` with Container Insights
- **ECS Service**: Fargate launch type with 1 task
- **Task Resources**: 256 CPU units, 512 MB memory
- **Container**: `kenesparta-app` on port 3000

### Load Balancing
- **ALB**: Public-facing Application Load Balancer
- **Target Group**: IP-based targeting for Fargate
- **Listeners**: HTTP (redirects to HTTPS), HTTPS with TLS 1.3
- **Health Checks**: HTTP on port 3000, path `/`

### Security
- **ALB Security Group**: Allows 80/443 from internet
- **ECS Tasks Security Group**: Allows 3000 from ALB only
- **IAM Roles**: OIDC federation for GitHub Actions
- **SSL/TLS**: ACM certificate for `*.kenesparta.dev`

### DNS & CDN
- **Route53**: A record alias to ALB
- **ACM**: Wildcard certificate with DNS validation

### Monitoring
- **CloudWatch Logs**: `/ecs/kenesparta-dev` (7-day retention)
- **Container Insights**: Enabled for ECS metrics
- **VPC Flow Logs**: 7-day retention for debugging

## CI/CD Pipeline

GitHub Actions workflow automatically:
1. Builds Docker image on push to `main` or version tags
2. Pushes image to AWS ECR
3. Updates ECS task definition
4. Deploys to ECS cluster with zero-downtime rolling update

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

- Single Fargate task (minimal resources)
- No CloudFront (simplified architecture)
- No NAT Gateway (using public subnets)
- ALB only (~$16-20/month vs CloudFront + ALB)

## License

This project is personal portfolio code. Feel free to reference the architecture and setup patterns.

## Contributing

This is a personal portfolio project, but feel free to open issues for bugs or suggestions.
