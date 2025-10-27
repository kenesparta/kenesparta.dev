resource "aws_route53_record" "proton_verify" {
  zone_id = local.zone_id
  name    = var.primary_dns
  type    = "TXT"
  ttl     = 300
  records = [
    "protonmail-verification=2c78b8bbf2a087f06e5d9fbe544b30bc8757dfd3",
    "v=spf1 include:_spf.protonmail.ch ~all",
  ]
}

resource "aws_route53_record" "proton_mx" {
  zone_id = local.zone_id
  name    = var.primary_dns
  type    = "MX"
  ttl     = 300
  records = [
    "10 mail.protonmail.ch",
    "20 mailsec.protonmail.ch"
  ]
}
resource "aws_route53_record" "proton_dkim" {
  zone_id = local.zone_id
  name    = "protonmail._domainkey.${var.primary_dns}"
  type    = "CNAME"
  ttl     = 300
  records = ["protonmail.domainkey.dfpms4bdksomvnzpy43pgng42puy72qrllingeu3xmgwiztwmacna.domains.proton.ch."]
}

resource "aws_route53_record" "proton_dkim_2" {
  zone_id = local.zone_id
  name    = "protonmail2._domainkey.${var.primary_dns}"
  type    = "CNAME"
  ttl     = 300
  records = ["protonmail2.domainkey.dfpms4bdksomvnzpy43pgng42puy72qrllingeu3xmgwiztwmacna.domains.proton.ch."]
}

resource "aws_route53_record" "proton_dkim_3" {
  zone_id = local.zone_id
  name    = "protonmail3._domainkey.${var.primary_dns}"
  type    = "CNAME"
  ttl     = 300
  records = ["protonmail3.domainkey.dfpms4bdksomvnzpy43pgng42puy72qrllingeu3xmgwiztwmacna.domains.proton.ch."]
}

resource "aws_route53_record" "proton_dmark" {
  zone_id = local.zone_id
  name    = "_dmarc.${var.primary_dns}"
  type    = "TXT"
  ttl     = 300
  records = ["v=DMARC1; p=quarantine"]
}
