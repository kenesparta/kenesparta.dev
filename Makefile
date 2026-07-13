.PHONY: dev/up dev/down dev/build dev/restart dev/logs dev/shell dev/clean css leptos/build

STYLE_DIR := apps/backend/style

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
