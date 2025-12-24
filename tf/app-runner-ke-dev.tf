# App Runner hosted zone IDs by region (AWS-managed, fixed values)
# https://docs.aws.amazon.com/general/latest/gr/apprunner.html
locals {
  apprunner_hosted_zone_ids = {
    "us-east-1" = "Z01915732ZBZKC8D32TPT"
  }
}

resource "aws_iam_role" "apprunner_access" {
  name = "kenesparta-apprunner-access"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "build.apprunner.amazonaws.com"
        }
      }
    ]
  })

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-apprunner-access-role"
    }
  )
}

resource "aws_iam_role_policy_attachment" "apprunner_access_ecr" {
  role       = aws_iam_role.apprunner_access.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSAppRunnerServicePolicyForECRAccess"
}

resource "aws_iam_role" "apprunner_instance" {
  name = "kenesparta-apprunner-instance"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "tasks.apprunner.amazonaws.com"
        }
      }
    ]
  })

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-apprunner-instance-role"
    }
  )
}

resource "aws_iam_role_policy" "apprunner_dynamodb" {
  name = "apprunner-dynamodb-policy"
  role = aws_iam_role.apprunner_instance.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowDynamoDBAccess"
        Effect = "Allow"
        Action = [
          "dynamodb:GetItem",
          "dynamodb:PutItem",
          "dynamodb:UpdateItem",
          "dynamodb:DeleteItem",
          "dynamodb:Query",
          "dynamodb:Scan",
          "dynamodb:BatchGetItem",
          "dynamodb:BatchWriteItem"
        ]
        Resource = [
          "arn:aws:dynamodb:${var.region}:*:table/kenesparta-blog-posts",
          "arn:aws:dynamodb:${var.region}:*:table/kenesparta-blog-posts/index/*"
        ]
      }
    ]
  })
}


resource "aws_apprunner_service" "kenesparta" {
  service_name = "kenesparta-dev"

  source_configuration {
    authentication_configuration {
      access_role_arn = aws_iam_role.apprunner_access.arn
    }

    image_repository {
      image_configuration {
        port = "3000"
        runtime_environment_variables = {
          LEPTOS_SITE_ADDR    = "0.0.0.0:3000"
          RUST_LOG            = "info"
          AWS_REGION          = var.region
          DYNAMODB_TABLE_NAME = "kenesparta-blog-posts"
        }
      }
      image_identifier      = "${aws_ecr_repository.kenesparta_app.repository_url}:latest"
      image_repository_type = "ECR"
    }

    auto_deployments_enabled = false
  }

  instance_configuration {
    cpu               = "256"
    memory            = "512"
    instance_role_arn = aws_iam_role.apprunner_instance.arn
  }

  health_check_configuration {
    protocol            = "HTTP"
    path                = "/"
    interval            = 10
    timeout             = 5
    healthy_threshold   = 1
    unhealthy_threshold = 5
  }

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-apprunner-service"
    }
  )
}

resource "aws_apprunner_custom_domain_association" "kenesparta" {
  domain_name          = var.primary_dns
  service_arn          = aws_apprunner_service.kenesparta.arn
  enable_www_subdomain = false
}

resource "aws_route53_record" "apprunner_main" {
  zone_id = local.zone_id
  name    = var.primary_dns
  type    = "A"

  alias {
    name                   = aws_apprunner_custom_domain_association.kenesparta.dns_target
    zone_id                = local.apprunner_hosted_zone_ids[var.region]
    evaluate_target_health = true
  }
}

# Certificate validation records for App Runner custom domain
resource "aws_route53_record" "apprunner_cert_validation" {
  for_each = {
    for r in aws_apprunner_custom_domain_association.kenesparta.certificate_validation_records : r.name => r
  }

  zone_id = local.zone_id
  name    = each.value.name
  type    = each.value.type
  ttl     = 300
  records = [each.value.value]
}
