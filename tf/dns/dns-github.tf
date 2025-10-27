resource "aws_route53_record" "github_pages_ipv6" {
  zone_id = local.zone_id
  name    = var.primary_dns
  type    = "AAAA"
  ttl     = 300
  records = [
    "2606:50c0:8000::153",
    "2606:50c0:8001::153",
    "2606:50c0:8002::153",
    "2606:50c0:8003::153"
  ]
}

resource "aws_route53_record" "github_pages_ipv4" {
  zone_id = local.zone_id
  name    = var.primary_dns
  type    = "A"
  ttl     = 300
  records = [
    "185.199.108.153",
    "185.199.109.153",
    "185.199.110.153",
    "185.199.111.153"
  ]
}

resource "aws_route53_record" "github_pages_challenge" {
  zone_id = local.zone_id
  name    = "_github-pages-challenge-kenesparta.${var.primary_dns}"
  type    = "TXT"
  ttl     = 300
  records = ["7d7d6c94b99633eb0ec4a287676023"]
}