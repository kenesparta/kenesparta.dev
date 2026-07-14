# ── Lightsail Container Service ──────────────────────────────────────────────
# Runs the site container pulled directly from the PUBLIC GHCR image
# (ghcr.io/kenesparta/kenespartadev). Unlike App Runner, Lightsail can pull from
# a third-party registry.
#
# Phase 1: additive only. App Runner (app-runner-ke-dev.tf) stays up until this
# is verified; the apex DNS is cut over to CloudFront -> Lightsail in a later
# step. No downtime.

resource "aws_lightsail_container_service" "app" {
  name        = "kenesparta-app"
  power       = "nano" # 0.25 vCPU / 512 MB, ~$7/mo
  scale       = 1
  is_disabled = false

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-lightsail"
    }
  )
}

# ── IAM user for DynamoDB access ─────────────────────────────────────────────
# Lightsail containers have no IAM role, so the app authenticates to DynamoDB
# with static keys passed as env vars. Scoped to the blog table only.
# TEMPORARY: remove once the blog moves to Postgres.
resource "aws_iam_user" "lightsail_app" {
  name = "kenesparta-lightsail-app"

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-lightsail-app"
    }
  )
}

resource "aws_iam_user_policy" "lightsail_dynamodb" {
  name = "dynamodb-blog-access"
  user = aws_iam_user.lightsail_app.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowDynamoDBBlogAccess"
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

resource "aws_iam_access_key" "lightsail_app" {
  user = aws_iam_user.lightsail_app.name
}

# ── Deployment (container spec: image, env, port, health check) ──────────────
resource "aws_lightsail_container_service_deployment_version" "app" {
  service_name = aws_lightsail_container_service.app.name

  container {
    container_name = "app"
    image          = "ghcr.io/kenesparta/kenespartadev:latest"

    ports = {
      "3000" = "HTTP"
    }

    environment = {
      LEPTOS_SITE_ADDR      = "0.0.0.0:3000"
      RUST_LOG              = "info"
      AWS_REGION            = var.region
      DYNAMODB_TABLE_NAME   = "kenesparta-blog-posts"
      AWS_ACCESS_KEY_ID     = aws_iam_access_key.lightsail_app.id
      AWS_SECRET_ACCESS_KEY = aws_iam_access_key.lightsail_app.secret
    }
  }

  public_endpoint {
    container_name = "app"
    container_port = 3000

    health_check {
      healthy_threshold   = 2
      unhealthy_threshold = 5
      timeout_seconds     = 5
      interval_seconds    = 10
      path                = "/"
      success_codes       = "200-399"
    }
  }
}

output "lightsail_app_url" {
  value       = aws_lightsail_container_service.app.url
  description = "Default HTTPS domain of the Lightsail container service (for verification and as the CloudFront origin)."
}
