SHELL:=/bin/bash
include .env

# Age key for the sops provider (decrypts ../secrets/prod.enc.env at
# plan/apply). sops' macOS default path differs, so pin it explicitly.
export SOPS_AGE_KEY_FILE ?= $(HOME)/.config/sops/age/keys.txt

login:
	@aws sso login --profile ${TF_VAR_aws_sso_profile}

fmt:
	@terraform fmt --recursive

dev/init: fmt
	@terraform init \
		-backend-config="bucket=tf.kenesparta.dev" \
		-backend-config="key=dns/prod/kenesparta.dev" \
		-backend-config="region=us-east-1" \
		-backend-config="profile=${TF_VAR_aws_sso_profile}"

dev/plan: dev/init
	@terraform plan -out create.tfplan

dev/apply: dev/plan
	@terraform apply create.tfplan

dev/destroy: dev/init
	@terraform destroy

# Rollout manual: despliega una versión YA publicada en ECR, sin tocar el
# resto de la infra (mismo apply acotado que hace el CI).
rollout: dev/init
	@test -n "$(VERSION)" || { echo "usage: make rollout VERSION=vX.Y.Z"; exit 1; }
	@terraform apply -auto-approve -input=false \
		-var="image_version=$(VERSION)" \
		-target=aws_lightsail_container_service_deployment_version.app