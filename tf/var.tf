variable "aws_sso_profile" {
  description = "AWS SSO profile for local runs (tf/.env). Empty in CI, where credentials come from the OIDC role via env vars."
  type        = string
  default     = ""
}

variable "image_version" {
  description = "Tag of the backend image in ECR to deploy (git tag vX.Y.Z). CI sets TF_VAR_image_version; a full local apply defaults to `latest`."
  type        = string
  default     = "latest"
}

variable "region" {
  type    = string
  default = "us-east-1"
}

variable "alias" {
  type    = string
  default = "us_east_1"
}

variable "project" {
  type    = string
  default = "dns"
}

variable "owner" {
  type    = string
  default = "kenesparta"
}

variable "primary_dns" {
  type    = string
  default = "kenesparta.dev"
}

variable "link_dns" {
  type    = string
  default = "kecc.link"
}

variable "environment" {
  description = "The type of deployment environment. Must be one of 'dev', or 'prod'."
  type        = string
  default     = "prod"
  validation {
    condition     = contains(["dev", "prod"], var.environment)
    error_message = "The 'environment_type' must be one of 'dev' or 'prod'."
  }
}
