variable "aws_sso_profile" {
  description = "(string) global project name"
  type        = string
}

variable "region" {
  type    = string
  default = "us-east-1"
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
