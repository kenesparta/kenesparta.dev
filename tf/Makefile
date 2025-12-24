SHELL:=/bin/bash
include .env

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

dev/apply-apprun:
	terraform apply -target=aws_apprunner_service.kenesparta \
                      -target=aws_apprunner_custom_domain_association.kenesparta \
                      -target=aws_iam_role.apprunner_access \
                      -target=aws_iam_role.apprunner_instance \
                      -target=aws_iam_role_policy.apprunner_dynamodb \
                      -target=aws_iam_role_policy_attachment.apprunner_access_ecr