SHELL:=/bin/bash
include .env

login:
	@aws sso login --profile ${TF_VAR_aws_sso_profile}

fmt:
	@terraform fmt --recursive

dev/init:
	@terraform init \
		-backend-config="bucket=tf.kenesparta.dev" \
		-backend-config="key=blog/prod/kenesparta.dev" \
		-backend-config="region=us-east-1" \
		-backend-config="profile=${TF_VAR_aws_sso_profile}"

dev/plan: dev/init
	terraform plan -out create.tfplan

dev/apply: dev/plan
	terraform apply create.tfplan

dev/destroy: dev/init
	terraform destroy
