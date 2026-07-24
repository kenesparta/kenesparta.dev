.PHONY: dev/up dev/down dev/build dev/restart dev/logs dev/shell dev/clean css leptos/build \
        secrets secrets-prod secrets-view secrets-rotate blog/ingest blog/publish cdn/upload

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

## blog/ingest:   renderiza content/posts/*.md y hace upsert en la BD (ENV=dev|prod)
##                PRUNE=1 borra además los posts de la BD sin archivo .md (espejo exacto)
blog/ingest:
	sops exec-env $(SECRETS) 'cargo run -p backend --no-default-features --features ingest --bin ingest -- content/posts $(if $(PRUNE),--prune)'

## blog/publish:  publica los posts en PRODUCCIÓN
blog/publish:
	$(MAKE) blog/ingest ENV=prod

# CDN: bucket S3 privado servido por CloudFront (tf/static-cdn.tf).
# El perfil AWS SSO sale de tf/.env (el mismo que usa Terraform).
-include tf/.env
CDN_BUCKET := cdn.kenesparta.dev
KEY ?= $(notdir $(FILE))

## cdn/upload:    sube un archivo al CDN y muestra su URL pública
##                FILE=<archivo local>  KEY=<ruta destino> (default: nombre del archivo)
##                ej: make cdn/upload FILE=arch.webp KEY=blog/mi-post/arch.webp
cdn/upload:
	@test -n "$(FILE)" || { echo 'usage: make cdn/upload FILE=<local-file> [KEY=<cdn-path>]'; exit 1; }
	@test -n "$(TF_VAR_aws_sso_profile)" || { echo 'TF_VAR_aws_sso_profile no definido: crea tf/.env (ver tf/Makefile)'; exit 1; }
	aws s3 cp "$(FILE)" "s3://$(CDN_BUCKET)/$(KEY)" --profile "$(TF_VAR_aws_sso_profile)"
	@echo "https://$(CDN_BUCKET)/$(KEY)"
