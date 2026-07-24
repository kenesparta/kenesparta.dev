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
  profile = var.aws_sso_profile
  region  = var.region
}

# Decrypts secrets/*.enc.env with the age key (SOPS_AGE_KEY_FILE, exported by
# the tf/Makefile). Runs locally at plan/apply time; CI never needs the key.
provider "sops" {}