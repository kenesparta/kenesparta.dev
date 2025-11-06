resource "aws_dynamodb_table" "blog_posts" {
  name         = "kenesparta-blog-posts"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "post_id"
  range_key    = "created_at"

  attribute {
    name = "post_id"
    type = "S"
  }

  attribute {
    name = "created_at"
    type = "N"
  }

  attribute {
    name = "status"
    type = "S"
  }

  global_secondary_index {
    name            = "StatusCreatedAtIndex"
    hash_key        = "status"
    range_key       = "created_at"
    projection_type = "ALL"
  }

  point_in_time_recovery {
    enabled = true
  }

  server_side_encryption {
    enabled = true
  }

  tags = {
    Name        = "kenesparta-blog-posts"
    Environment = "production"
    Project     = "kenesparta.dev"
  }
}
