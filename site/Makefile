.PHONY: dev/up dev/down dev/build dev/restart dev/logs dev/shell dev/clean

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
