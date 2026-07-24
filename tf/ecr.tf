# ── ECR (private image registry) ─────────────────────────────────────────────
# The backend image lives in a private ECR repo. Lightsail pulls it through the
# service's ECR image-puller role (private_registry_access in lightsail.tf);
# the repository policy below grants that role the pull actions. CI pushes
# `vX.Y.Z` + `latest` on every version tag (publish-image.yml).

resource "aws_ecr_repository" "app" {
  name = "kenespartadev"
  # MUTABLE so CI can move the `latest` alias alongside the vX.Y.Z tags
  # (a full local apply defaults to `latest`, see var.image_version).
  image_tag_mutability = "MUTABLE"

  image_scanning_configuration {
    scan_on_push = true
  }

  tags = merge(
    local.common_tags,
    {
      Name = "kenespartadev-ecr"
    }
  )
}

# Keep storage bounded (~$0.10/GB/mo): only the most recent images survive.
resource "aws_ecr_lifecycle_policy" "app" {
  repository = aws_ecr_repository.app.name

  policy = jsonencode({
    rules = [
      {
        rulePriority = 1
        description  = "keep only the last 10 images"
        selection = {
          tagStatus   = "any"
          countType   = "imageCountMoreThan"
          countNumber = 10
        }
        action = { type = "expire" }
      }
    ]
  })
}

# Grant the Lightsail image-puller principal read access to this repo.
resource "aws_ecr_repository_policy" "lightsail_pull" {
  repository = aws_ecr_repository.app.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "AllowLightsailPull"
        Effect = "Allow"
        Principal = {
          AWS = aws_lightsail_container_service.app.private_registry_access[0].ecr_image_puller_role[0].principal_arn
        }
        Action = [
          "ecr:BatchGetImage",
          "ecr:GetDownloadUrlForLayer"
        ]
      }
    ]
  })
}
