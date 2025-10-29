variable "aws_sso_profile" {
  description = "(string) global project name"
  type        = string
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
