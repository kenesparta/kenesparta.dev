# Ken Esparta — Personal Website

Live: https://kenesparta.dev

A fast, lightweight personal website built with Rust, Leptos, and Trunk. It compiles to WebAssembly (WASM) and ships a minimal, static bundle for performance and reliability.

## Features
- Rust + Leptos frontend compiled to WASM
- Simple component structure (Header, Social Links, Home page)
- Fast local development with Trunk (auto-reload)
- Production build with minification and content hashing
- Ready for GitHub Pages (CNAME included)

## Tech Stack
- Rust (stable)
- Leptos (client-side)
- Trunk (build/serve)
- WebAssembly (wasm32-unknown-unknown target)
- Vanilla CSS (no framework)

## Prerequisites
- Rust toolchain: https://rustup.rs
- wasm target: wasm32-unknown-unknown
- Trunk: https://trunkrs.dev

Install prerequisites:
- rustup target add wasm32-unknown-unknown
- cargo install trunk

## Getting Started
1. Clone the repo
   - git clone https://github.com/kenesparta/kenesparta.dev.git
   - cd kenesparta.dev
2. Install prerequisites (see above)
3. Start the dev server
   - trunk serve
   - Open http://localhost:8080

Trunk is configured to serve on port 8080 (see Trunk.toml). Hot reload is enabled by default.

## Build
- Using Trunk
  - trunk build
  - Output will be in the dist/ directory
- Using Make
  - make build

Production builds are minified and hashed per Trunk.toml.

## Deployment
This repository is set up for static hosting (e.g., GitHub Pages, Cloudflare Pages, Netlify). A CNAME file is included for kenesparta.dev.

Typical GitHub Pages deployment options:
- Using an actions workflow that builds with Trunk and publishes dist/ to the gh-pages branch
- Or build locally and push the dist/ contents to gh-pages

Ensure your DNS points to your hosting provider and the CNAME file remains at the repository root in the published artifact.

## Project Structure
- src/components
  - header.rs — Top navigation/header
  - social_links.rs — External links (GitHub, LinkedIn)
- src/pages
  - home.rs — Landing page with profile and summary
- assets/css — Component/page styles
- assets/img — Images and icons
- index.html — Entry HTML used by Trunk
- Trunk.toml — Build/serve configuration
- Makefile — Convenience build target(s)

## Troubleshooting
- Trunk not found
  - Install with: cargo install trunk
- wasm target missing
  - rustup target add wasm32-unknown-unknown
- Port already in use
  - Edit serve.port in Trunk.toml or run: trunk serve --port 8081

## License
MIT © 2025 Ken Esparta Ccorahua
