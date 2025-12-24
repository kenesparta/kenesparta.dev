resource "aws_iam_openid_connect_provider" "github" {
  url            = "https://token.actions.githubusercontent.com"
  client_id_list = ["sts.amazonaws.com"]
  thumbprint_list = [
    "6938fd4d98bab03faadb97b34396831e3780aea1",
    "1b511abead59c6ce207077c0bf0e0043b1382612"
  ]

  tags = merge(
    local.common_tags,
    {
      Name = "github-actions-oidc-provider"
    }
  )
}

resource "aws_iam_role" "github_actions_deploy" {
  name = "github-actions-ecr-ecs-deploy"
  path = "/github-actions/"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowGitHubActionsOIDC"
        Effect = "Allow"
        Principal = {
          Federated = aws_iam_openid_connect_provider.github.arn
        }
        Action = "sts:AssumeRoleWithWebIdentity"
        Condition = {
          StringEquals = {
            "token.actions.githubusercontent.com:aud" = "sts.amazonaws.com"
          }
          StringLike = {
            "token.actions.githubusercontent.com:sub" = [
              "repo:kenesparta/kenesparta.dev:ref:refs/heads/main",
              "repo:kenesparta/kenesparta.dev:ref:refs/tags/*",
              "repo:kenesparta/typst-resume:ref:refs/heads/main",
              "repo:kenesparta/typst-resume:ref:refs/tags/*",
            ]
          }
        }
      }
    ]
  })

  tags = merge(
    local.common_tags,
    {
      Name        = "github-actions-ecr-ecs-deploy"
      Description = "Role for GitHub Actions to deploy to ECR and ECS"
    }
  )
}

resource "aws_iam_role_policy" "github_actions_ecr" {
  name = "ecr-push-policy"
  role = aws_iam_role.github_actions_deploy.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowECRAuth"
        Effect = "Allow"
        Action = [
          "ecr:GetAuthorizationToken"
        ]
        Resource = "*"
      },
      {
        Sid    = "AllowECRImageManagement"
        Effect = "Allow"
        Action = [
          "ecr:BatchCheckLayerAvailability",
          "ecr:GetDownloadUrlForLayer",
          "ecr:BatchGetImage",
          "ecr:PutImage",
          "ecr:InitiateLayerUpload",
          "ecr:UploadLayerPart",
          "ecr:CompleteLayerUpload"
        ]
        Resource = "arn:aws:ecr:${var.region}:*:repository/*"
      }
    ]
  })
}

resource "aws_iam_role_policy" "github_actions_apprunner" {
  name = "apprunner-deploy-policy"
  role = aws_iam_role.github_actions_deploy.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowAppRunnerDeployment"
        Effect = "Allow"
        Action = [
          "apprunner:StartDeployment",
          "apprunner:DescribeService"
        ]
        Resource = "arn:aws:apprunner:${var.region}:*:service/kenesparta-dev/*"
      }
    ]
  })
}

resource "aws_iam_role_policy" "github_actions_s3" {
  name = "cdn-bucket-write-policy"
  role = aws_iam_role.github_actions_deploy.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowS3BucketWrite"
        Effect = "Allow"
        Action = [
          "s3:PutObject",
          "s3:PutObjectAcl",
          "s3:DeleteObject"
        ]
        Resource = "${aws_s3_bucket.cdn_bucket.arn}/*"
      },
      {
        Sid    = "AllowS3BucketList"
        Effect = "Allow"
        Action = [
          "s3:ListBucket",
          "s3:GetBucketLocation"
        ]
        Resource = aws_s3_bucket.cdn_bucket.arn
      }
    ]
  })
}
