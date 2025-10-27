terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "5.89.0"
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