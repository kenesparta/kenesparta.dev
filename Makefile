.PHONY: dev/up dev/down dev/build dev/restart dev/logs dev/shell dev/clean css leptos/build \
        secrets secrets-prod secrets-view secrets-rotate blog/build blog/ingest blog/publish cdn/upload

STYLE_DIR := apps/backend/style

# Secrets: sops + age. ENV selects the file (dev|prod).
ENV ?= dev
SECRETS := secrets/$(ENV).enc.env
# sops' default age key path on macOS is ~/Library/Application Support/...;
# we keep the key at the XDG path, so point sops (and Terraform) at it.
export SOPS_AGE_KEY_FILE ?= $(HOME)/.config/sops/age/keys.txt

# Development commands
dev/up: ## Start development environment with hot-reload
	docker compose -f docker-compose.dev.yml up

dev/down: ## Stop development environment
	docker compose -f docker-compose.dev.yml down

dev/build: ## Rebuild development Docker image
	docker compose -f docker-compose.dev.yml build

dev/restart: ## Restart development environment
	docker compose -f docker-compose.dev.yml restart

dev/shell: ## Open shell in development container
	docker compose -f docker-compose.dev.yml exec leptos-dev /bin/bash

dev/clean: ## Remove development volumes and containers
	docker compose -f docker-compose.dev.yml down -v

prod/run:
	docker compose up -d

# Styles: concatenate style/parts/*.css (the source) into the single main.css
# bundle cargo-leptos serves. main.css is GENERATED — edit files in parts/.
# Run this after editing a part; cargo-leptos watches main.css, not the parts.
css: ## Rebuild the CSS bundle from style/parts/*.css
	@printf '/* GENERATED from style/parts/ by "make css" - do not edit; edit the parts. */\n\n' > $(STYLE_DIR)/main.css
	@cat $(STYLE_DIR)/parts/*.css >> $(STYLE_DIR)/main.css
	@echo "Rebuilt $(STYLE_DIR)/main.css from parts/"

leptos/build: css
	cd apps/backend && cargo leptos build --release

## secrets:       edita los secretos de dev con el editor ($$EDITOR), re-cifra al guardar
secrets:
	@sops secrets/dev.enc.env

## secrets-prod:  edita los secretos de prod
secrets-prod:
	@sops secrets/prod.enc.env

## secrets-view:  muestra los secretos descifrados en stdout (ENV=dev|prod)
secrets-view:
	@sops -d $(SECRETS)

## secrets-rotate: re-cifra el data key para los destinatarios actuales de .sops.yaml
##                 (tras añadir/quitar una public key age)
secrets-rotate:
	@sops updatekeys secrets/dev.enc.env secrets/prod.enc.env

# The ingest binary is compiled with NO secret in its environment, and only
# then executed under `sops exec-env`. A single `cargo run` under sops would
# hand DATABASE_URL to every build.rs and proc-macro in the dependency graph
# while it compiles (SEC-001 in SECURITY.md). `env -u` also drops a
# DATABASE_URL inherited from the shell profile, so the build sees none at all;
# `--locked` refuses to rewrite Cargo.lock on the side, so the binary only ever
# links what the committed lockfile pins.
INGEST_BIN := $(or $(CARGO_TARGET_DIR),target)/debug/ingest

## blog/build:    compila el CLI de ingest sin ningún secreto en el entorno (SEC-001)
blog/build:
	env -u DATABASE_URL cargo build --locked -p backend --no-default-features --features ingest --bin ingest

## blog/ingest:   renderiza content/posts/*.md y hace upsert en la BD (ENV=dev|prod)
##                PRUNE=1 borra además los posts de la BD sin archivo .md (espejo exacto)
blog/ingest: blog/build
	sops exec-env $(SECRETS) '$(INGEST_BIN) content/posts $(if $(PRUNE),--prune)'

## blog/publish:  publica los posts en PRODUCCIÓN (Aurora DSQL, personal-infra §5.22)
##                Aplica las migraciones pendientes y hace upsert de content/posts.
##                Sin secretos ni túnel: el token IAM se firma con tu perfil SSO
##                y se conecta como blog_owner (mapeado a tu rol por
##                `make dsql/roles` en personal-infra). PRUNE=1 también borra.
# The endpoint and the SSO profile are read from personal-infra, the repository
# that owns them, so neither is written here: the profile name carries the
# account id (SECURITY.md SEC-025). The binary is still built first with no
# secret in its environment (blog/build, SEC-001), and runs with DATABASE_URL
# unset so a stray shell variable cannot redirect it.
PUBLISH_INFRA   ?= ../personal-infra
PUBLISH_PROFILE ?= $(shell sed -n 's/^export TF_VAR_aws_sso_profile=//p' $(PUBLISH_INFRA)/terraform/.env 2>/dev/null)

blog/publish: blog/build
	@test -n "$(PUBLISH_PROFILE)" || { echo "no SSO profile: $(PUBLISH_INFRA)/terraform/.env not found"; exit 1; }
	@endpoint="$$(terraform -chdir=$(PUBLISH_INFRA)/terraform output -json dsql_endpoints | jq -r '.blog // empty')"; \
	test -n "$$endpoint" || { echo "no DSQL endpoint for blog in $(PUBLISH_INFRA)'s outputs"; exit 1; }; \
	AWS_PROFILE='$(PUBLISH_PROFILE)' DSQL_ENDPOINT="$$endpoint" DSQL_USER=blog_owner DSQL_SCHEMA=blog \
	  env -u DATABASE_URL $(INGEST_BIN) content/posts $(if $(PRUNE),--prune)
