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

resource "aws_iam_role_policy" "ecs_task_dynamodb" {
  name = "ecs-task-dynamodb-policy"
  role = aws_iam_role.ecs_task.id

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

resource "aws_security_group" "alb" {
  name        = "kenesparta-alb-sg"
  description = "Security group for Application Load Balancer"
  vpc_id      = aws_vpc.main.id

  ingress {
    description = "Allow HTTP from internet"
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    description = "Allow HTTPS from internet"
    from_port   = 443
    to_port     = 443
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
      Name = "kenesparta-alb-sg"
    }
  )
}

resource "aws_security_group" "ecs_tasks" {
  name        = "kenesparta-ecs-tasks-sg"
  description = "Security group for ECS tasks"
  vpc_id      = aws_vpc.main.id

  ingress {
    description     = "Allow traffic from ALB"
    from_port       = 3000
    to_port         = 3000
    protocol        = "tcp"
    security_groups = [aws_security_group.alb.id]
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

# Application Load Balancer
resource "aws_lb" "kenesparta" {
  name               = "kenesparta-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets = [
    aws_subnet.public_1.id,
    aws_subnet.public_2.id,
    aws_subnet.public_3.id
  ]

  enable_deletion_protection       = false
  enable_http2                     = true
  enable_cross_zone_load_balancing = true

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-alb"
    }
  )
}

# Target Group for ECS tasks
resource "aws_lb_target_group" "kenesparta" {
  name        = "kenesparta-tg"
  port        = 3000
  protocol    = "HTTP"
  vpc_id      = aws_vpc.main.id
  target_type = "ip"

  health_check {
    enabled             = true
    healthy_threshold   = 2
    interval            = 30
    matcher             = "200"
    path                = "/"
    port                = "traffic-port"
    protocol            = "HTTP"
    timeout             = 5
    unhealthy_threshold = 3
  }

  deregistration_delay = 30

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-target-group"
    }
  )
}

resource "aws_lb_listener" "http" {
  load_balancer_arn = aws_lb.kenesparta.arn
  port              = "80"
  protocol          = "HTTP"

  default_action {
    type = "redirect"

    redirect {
      port        = "443"
      protocol    = "HTTPS"
      status_code = "HTTP_301"
    }
  }
}

resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.kenesparta.arn
  port              = "443"
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = aws_acm_certificate.kenesparta_cert.arn

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.kenesparta.arn
  }

  depends_on = [aws_acm_certificate_validation.kenesparta_cert]
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
        },
        {
          name  = "AWS_REGION"
          value = var.region
        },
        {
          name  = "DYNAMODB_TABLE_NAME"
          value = "kenesparta-blog-posts"
        }
      ]
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
  name                 = "kenesparta-service"
  cluster              = aws_ecs_cluster.kenesparta.id
  task_definition      = aws_ecs_task_definition.kenesparta.arn
  desired_count        = 1
  launch_type          = "FARGATE"
  force_new_deployment = true

  network_configuration {
    subnets = [
      aws_subnet.public_1.id,
      aws_subnet.public_2.id,
      aws_subnet.public_3.id
    ]
    security_groups  = [aws_security_group.ecs_tasks.id]
    assign_public_ip = true
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.kenesparta.arn
    container_name   = "kenesparta-app"
    container_port   = 3000
  }

  depends_on = [
    aws_iam_role_policy_attachment.ecs_task_execution,
    aws_lb_listener.https
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
    name                   = aws_lb.kenesparta.dns_name
    zone_id                = aws_lb.kenesparta.zone_id
    evaluate_target_health = true
  }
}
