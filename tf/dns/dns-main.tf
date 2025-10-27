resource "aws_route53_zone" "kenespartadev" {
  name = var.primary_dns
  tags = merge(
    local.common_tags,
    {
      Name = "main-DNS"
    }
  )
}

resource "aws_route53_hosted_zone_dnssec" "kenespartadev" {
  hosted_zone_id = aws_route53_zone.kenespartadev.id
  depends_on     = [aws_route53_key_signing_key.kenespartadev]
}

resource "aws_route53_key_signing_key" "kenespartadev" {
  name                       = "kenespartadev"
  hosted_zone_id             = aws_route53_zone.kenespartadev.id
  key_management_service_arn = aws_kms_key.kenespartadev_key_dnssec.arn
  status                     = "ACTIVE"
}

data "aws_caller_identity" "current" {}

resource "aws_kms_key" "kenespartadev_key_dnssec" {
  customer_master_key_spec = "ECC_NIST_P256"
  deletion_window_in_days  = 7
  key_usage                = "SIGN_VERIFY"
  policy = jsonencode({
    Statement = [
      {
        Action = [
          "kms:DescribeKey",
          "kms:GetPublicKey",
          "kms:Sign",
        ],
        Effect = "Allow",
        Principal = {
          Service = "dnssec-route53.amazonaws.com"
        },
        Resource = "*",
        Sid      = "Allow Route53 DNSSEC Service"
      },
      {
        Action = "kms:*",
        Effect = "Allow",
        Principal = {
          AWS = "arn:aws:iam::${data.aws_caller_identity.current.account_id}:root"
        },
        Resource = "*",
        Sid      = "Allow administration of the key"
      }
    ],
    Version = "2012-10-17"
  })
}
