resource "aws_acm_certificate" "kenesparta_cert" {
  domain_name               = var.primary_dns
  subject_alternative_names = ["*.${var.primary_dns}"]
  validation_method         = "DNS"

  lifecycle {
    create_before_destroy = true
  }

  tags = merge(
    local.common_tags,
    {
      Name = var.primary_dns
    }
  )
}

resource "aws_acm_certificate_validation" "kenesparta_cert" {
  certificate_arn         = aws_acm_certificate.kenesparta_cert.arn
  validation_record_fqdns = [for record in aws_route53_record.kenesparta_cert_validation : record.fqdn]
}

resource "aws_route53_record" "kenesparta_cert_validation" {
  for_each = {
    for dvo in aws_acm_certificate.kenesparta_cert.domain_validation_options : dvo.domain_name => {
      name   = dvo.resource_record_name
      record = dvo.resource_record_value
      type   = dvo.resource_record_type
    }
  }

  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = local.zone_id
}
