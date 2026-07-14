# ── CloudFront in front of the Lightsail container service ───────────────────
# Route53 cannot ALIAS the apex directly to a Lightsail container service, so
# CloudFront fronts it: apex ALIAS -> CloudFront -> Lightsail origin (HTTPS).
# Reuses the existing ACM cert (acm.tf). Pure pass-through (no caching) to keep
# SSR responses fresh and avoid stale assets; a caching behavior for /pkg/* can
# be added later once assets are content-hashed.

locals {
  # Strip scheme + trailing slash from the Lightsail service URL to get the
  # bare hostname CloudFront uses as its origin.
  lightsail_origin_domain = trimsuffix(trimprefix(aws_lightsail_container_service.app.url, "https://"), "/")
}

data "aws_cloudfront_cache_policy" "caching_disabled" {
  name = "Managed-CachingDisabled"
}

# Forwards all viewer headers/cookies/query strings to the origin EXCEPT Host,
# so CloudFront presents the Lightsail origin domain as Host (correct for a
# custom origin). Server functions (POST) and SSR need everything else.
data "aws_cloudfront_origin_request_policy" "all_viewer_except_host" {
  name = "Managed-AllViewerExceptHostHeader"
}

resource "aws_cloudfront_distribution" "app" {
  enabled         = true
  is_ipv6_enabled = true
  comment         = "kenesparta.dev -> Lightsail container (GHCR image)"
  aliases         = [var.primary_dns]
  price_class     = "PriceClass_100"

  origin {
    origin_id   = "lightsail"
    domain_name = local.lightsail_origin_domain

    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  default_cache_behavior {
    target_origin_id         = "lightsail"
    viewer_protocol_policy   = "redirect-to-https"
    allowed_methods          = ["GET", "HEAD", "OPTIONS", "PUT", "POST", "PATCH", "DELETE"]
    cached_methods           = ["GET", "HEAD"]
    cache_policy_id          = data.aws_cloudfront_cache_policy.caching_disabled.id
    origin_request_policy_id = data.aws_cloudfront_origin_request_policy.all_viewer_except_host.id
    compress                 = true
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    acm_certificate_arn      = aws_acm_certificate.kenesparta_cert.arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.2_2021"
  }

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-cloudfront"
    }
  )
}

# ── Apex cutover: kenesparta.dev A ALIAS -> CloudFront (was App Runner) ───────
# `moved` reuses the SAME Route53 record (atomic UPSERT, no downtime) instead of
# destroy+create when renaming it off the App Runner resource.
moved {
  from = aws_route53_record.apprunner_main
  to   = aws_route53_record.apex_cloudfront
}

resource "aws_route53_record" "apex_cloudfront" {
  zone_id = local.zone_id
  name    = var.primary_dns
  type    = "A"

  alias {
    name                   = aws_cloudfront_distribution.app.domain_name
    zone_id                = "Z2FDTNDATAQYW2" # CloudFront's fixed global hosted zone id
    evaluate_target_health = false
  }
}

output "cloudfront_domain" {
  value       = aws_cloudfront_distribution.app.domain_name
  description = "CloudFront distribution domain (the apex aliases to this)."
}
