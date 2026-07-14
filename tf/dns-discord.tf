resource "aws_route53_record" "discord_verify" {
  zone_id = local.zone_id
  name    = "_discord.${var.primary_dns}"
  type    = "TXT"
  ttl     = 300
  records = ["dh=d83cef255b6987c44589d409c22b0146ad3a0793"]
}
