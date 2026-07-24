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

# NOTE: the role name ("...ecr-ecs-deploy") predates the Lightsail migration
# and is kept so the AWS_ROLE_ARN secret / OIDC binding stay valid; renaming it
# would change the ARN. (Fittingly, ECR is back in use since the move from
# public GHCR to the private ECR repo.)

# Lets the CI roll out a new Lightsail container deployment (terraform apply
# -target of the deployment resource). Lightsail container-service actions
# don't support resource-level ARNs, so Resource must be "*".
resource "aws_iam_role_policy" "github_actions_lightsail" {
  name = "lightsail-deploy-policy"
  role = aws_iam_role.github_actions_deploy.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowLightsailContainerDeploy"
        Effect = "Allow"
        Action = [
          "lightsail:GetContainerServices",
          "lightsail:GetContainerServiceDeployments",
          "lightsail:CreateContainerServiceDeployment"
        ]
        Resource = "*"
      }
    ]
  })
}

# ECR push for the CI build-push job. GetAuthorizationToken cannot be scoped
# to a repository; the push/pull actions are limited to the app repo.
resource "aws_iam_role_policy" "github_actions_ecr" {
  name = "ecr-push-policy"
  role = aws_iam_role.github_actions_deploy.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid      = "AllowEcrAuth"
        Effect   = "Allow"
        Action   = ["ecr:GetAuthorizationToken"]
        Resource = "*"
      },
      {
        Sid    = "AllowEcrPushPull"
        Effect = "Allow"
        Action = [
          "ecr:BatchCheckLayerAvailability",
          "ecr:BatchGetImage",
          "ecr:GetDownloadUrlForLayer",
          "ecr:InitiateLayerUpload",
          "ecr:UploadLayerPart",
          "ecr:CompleteLayerUpload",
          "ecr:PutImage",
          "ecr:DescribeRepositories",
          "ecr:ListTagsForResource"
        ]
        Resource = aws_ecr_repository.app.arn
      }
    ]
  })
}

# Terraform state access for the CI deploy job (init + apply -target of the
# Lightsail deployment): only the state object of this stack, nothing else.
resource "aws_iam_role_policy" "github_actions_tfstate" {
  name = "tf-state-policy"
  role = aws_iam_role.github_actions_deploy.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid      = "AllowStateList"
        Effect   = "Allow"
        Action   = ["s3:ListBucket"]
        Resource = "arn:aws:s3:::tf.kenesparta.dev"
      },
      {
        Sid      = "AllowStateReadWrite"
        Effect   = "Allow"
        Action   = ["s3:GetObject", "s3:PutObject", "s3:DeleteObject"]
        Resource = "arn:aws:s3:::tf.kenesparta.dev/dns/prod/kenesparta.dev*"
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
