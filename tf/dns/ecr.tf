resource "aws_ecr_repository" "kenesparta_app" {
  name                 = "kenesparta-dev"
  image_tag_mutability = "MUTABLE"

  image_scanning_configuration {
    scan_on_push = true
  }

  tags = merge(
    local.common_tags,
    {
      Name = "kenesparta-dev-ecr"
    }
  )
}

resource "aws_ecr_lifecycle_policy" "kenesparta_app" {
  repository = aws_ecr_repository.kenesparta_app.name

  policy = jsonencode({
    rules = [
      {
        rulePriority = 1
        description  = "Keep last 10 images"
        selection = {
          tagStatus   = "any"
          countType   = "imageCountMoreThan"
          countNumber = 10
        }
        action = {
          type = "expire"
        }
      }
    ]
  })
}
