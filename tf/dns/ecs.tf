resource "aws_ecs_cluster" "kenesparta" {
  name = "kenesparta-cluster"

  setting {
    name  = "containerInsights"
    value = "enabled"
  }

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-ecs-cluster"
    }
  )
}

resource "aws_cloudwatch_log_group" "kenesparta_app" {
  name              = "/ecs/kenesparta-dev"
  retention_in_days = 7

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-app-logs"
    }
  )
}

resource "aws_iam_role" "ecs_task_execution" {
  name = "kenesparta-ecs-task-execution"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        }
      }
    ]
  })

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-ecs-task-execution-role"
    }
  )
}

resource "aws_iam_role_policy_attachment" "ecs_task_execution" {
  role       = aws_iam_role.ecs_task_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_iam_role" "ecs_task" {
  name = "kenesparta-ecs-task"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        }
      }
    ]
  })

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-ecs-task-role"
    }
  )
}

resource "aws_security_group" "ecs_tasks" {
  name        = "kenesparta-ecs-tasks-sg"
  description = "Security group for ECS tasks"
  vpc_id      = aws_vpc.main.id

  ingress {
    description = "Allow HTTP traffic from CloudFront and internet"
    from_port   = 3000
    to_port     = 3000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    description = "Allow all outbound"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-ecs-tasks-sg"
    }
  )
}

resource "aws_service_discovery_public_dns_namespace" "kenesparta" {
  name        = "ecs.${var.primary_dns}"
  description = "Public DNS namespace for ECS service discovery"

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-service-discovery"
    }
  )
}

resource "aws_service_discovery_service" "kenesparta_app" {
  name = "app"

  dns_config {
    namespace_id = aws_service_discovery_public_dns_namespace.kenesparta.id

    dns_records {
      ttl  = 10
      type = "A"
    }

    routing_policy = "MULTIVALUE"
  }

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-app-discovery"
    }
  )
}

# CloudFront Origin Access Control for ECS
resource "aws_cloudfront_cache_policy" "kenesparta_app" {
  name        = "kenesparta-app-cache-policy"
  comment     = "Cache policy for Kenesparta app"
  default_ttl = 0
  max_ttl     = 31536000
  min_ttl     = 0

  parameters_in_cache_key_and_forwarded_to_origin {
    cookies_config {
      cookie_behavior = "all"
    }

    headers_config {
      header_behavior = "whitelist"
      headers {
        items = ["Host", "Origin", "Referer"]
      }
    }

    query_strings_config {
      query_string_behavior = "all"
    }

    enable_accept_encoding_brotli = true
    enable_accept_encoding_gzip   = true
  }
}

# CloudFront Origin Request Policy
resource "aws_cloudfront_origin_request_policy" "kenesparta_app" {
  name    = "kenesparta-app-origin-policy"
  comment = "Origin request policy for Kenesparta app"

  cookies_config {
    cookie_behavior = "all"
  }

  headers_config {
    header_behavior = "allViewer"
  }

  query_strings_config {
    query_string_behavior = "all"
  }
}

resource "aws_cloudfront_distribution" "kenesparta_app" {
  enabled             = true
  is_ipv6_enabled     = true
  comment             = "CloudFront distribution for kenesparta.dev"
  default_root_object = ""
  price_class         = "PriceClass_100"
  aliases             = [var.primary_dns]

  origin {
    domain_name = "app.${aws_service_discovery_public_dns_namespace.kenesparta.name}"
    origin_id   = "ecs-kenesparta-app"

    custom_origin_config {
      http_port              = 3000
      https_port             = 443
      origin_protocol_policy = "http-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  default_cache_behavior {
    allowed_methods          = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods           = ["GET", "HEAD"]
    target_origin_id         = "ecs-kenesparta-app"
    cache_policy_id          = aws_cloudfront_cache_policy.kenesparta_app.id
    origin_request_policy_id = aws_cloudfront_origin_request_policy.kenesparta_app.id

    viewer_protocol_policy = "redirect-to-https"
    compress               = true
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

  depends_on = [aws_acm_certificate_validation.kenesparta_cert]

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-app-distribution"
    }
  )
}

resource "aws_ecs_task_definition" "kenesparta" {
  family                   = "kenesparta-dev"
  requires_compatibilities = ["FARGATE"]
  network_mode             = "awsvpc"
  cpu                      = "256"
  memory                   = "512"
  execution_role_arn       = aws_iam_role.ecs_task_execution.arn
  task_role_arn            = aws_iam_role.ecs_task.arn

  container_definitions = jsonencode([
    {
      name      = "kenesparta-app"
      image     = "${aws_ecr_repository.kenesparta_app.repository_url}:latest"
      essential = true

      portMappings = [
        {
          containerPort = 3000
          hostPort      = 3000
          protocol      = "tcp"
        }
      ]

      environment = [
        {
          name  = "LEPTOS_SITE_ADDR"
          value = "0.0.0.0:3000"
        },
        {
          name  = "RUST_LOG"
          value = "info"
        }
      ]

      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = aws_cloudwatch_log_group.kenesparta_app.name
          "awslogs-region"        = var.region
          "awslogs-stream-prefix" = "ecs"
        }
      }
    }
  ])

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-task-definition"
    }
  )
}

resource "aws_ecs_service" "kenesparta" {
  name            = "kenesparta-service"
  cluster         = aws_ecs_cluster.kenesparta.id
  task_definition = aws_ecs_task_definition.kenesparta.arn
  desired_count   = 1
  launch_type     = "FARGATE"

  network_configuration {
    subnets = [
      aws_subnet.public_1.id,
      aws_subnet.public_2.id,
      aws_subnet.public_3.id
    ]
    security_groups  = [aws_security_group.ecs_tasks.id]
    assign_public_ip = true
  }

  service_registries {
    registry_arn = aws_service_discovery_service.kenesparta_app.arn
  }

  depends_on = [
    aws_iam_role_policy_attachment.ecs_task_execution
  ]

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-ecs-service"
    }
  )
}

resource "aws_route53_record" "kenesparta_main" {
  zone_id = local.zone_id
  name    = var.primary_dns
  type    = "A"

  alias {
    name                   = aws_cloudfront_distribution.kenesparta_app.domain_name
    zone_id                = aws_cloudfront_distribution.kenesparta_app.hosted_zone_id
    evaluate_target_health = false
  }
}
