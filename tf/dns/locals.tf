locals {
  common_tags = {
    Project     = var.project
    Owner       = var.owner
    Environment = var.environment
    ManagedBy   = "Terraform"
  }

  zone_id = aws_route53_zone.kenespartadev.zone_id

  cdn_main_bucket = "cdn.${var.primary_dns}"
}
