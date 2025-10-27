resource "aws_iam_user" "github_actions_cdn" {
  name = "github-actions-cdn-writer"
  path = "/service-accounts/"

  tags = merge(
    local.common_tags,
    {
      Name        = "github-actions-cdn-writer"
      Description = "Service account for GitHub Actions to write to CDN bucket"
    }
  )
}

resource "aws_iam_user_policy" "github_actions_cdn_write" {
  name = "cdn-bucket-write-policy"
  user = aws_iam_user.github_actions_cdn.name

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
