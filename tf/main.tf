terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "6.15.0"
    }
    sops = {
      source  = "carlpett/sops"
      version = "~> 1.2"
    }
  }
  backend "s3" {
    encrypt = true
  }
}

provider "aws" {
  # Local runs use the SSO profile (tf/.env); CI leaves it empty and
  # authenticates with the OIDC role's env credentials.
  profile = var.aws_sso_profile != "" ? var.aws_sso_profile : null
  region  = var.region
}

# Decrypts secrets/*.enc.env with the age key: locally via SOPS_AGE_KEY_FILE
# (exported by the tf/Makefile), in CI via the SOPS_AGE_KEY repo secret
# (the deploy job needs it to render the container env vars).
provider "sops" {}