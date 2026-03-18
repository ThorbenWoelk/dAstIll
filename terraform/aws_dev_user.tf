resource "aws_iam_user" "dev" {
  name = "${var.app_name}-dev"
  tags = { App = var.app_name }
}

resource "aws_iam_access_key" "dev" {
  user = aws_iam_user.dev.name
}

resource "aws_iam_user_policy" "dev_s3_data" {
  name = "${var.app_name}-dev-s3-data"
  user = aws_iam_user.dev.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:DeleteObject",
          "s3:ListBucket",
        ]
        Resource = [
          aws_s3_bucket.data.arn,
          "${aws_s3_bucket.data.arn}/*",
        ]
      }
    ]
  })
}

resource "aws_iam_user_policy" "dev_s3_vectors" {
  name = "${var.app_name}-dev-s3-vectors"
  user = aws_iam_user.dev.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3vectors:PutVectors",
          "s3vectors:GetVectors",
          "s3vectors:DeleteVectors",
          "s3vectors:QueryVectors",
          "s3vectors:ListVectors",
          "s3vectors:GetVectorBucket",
          "s3vectors:ListVectorIndexes",
          "s3vectors:GetIndex",
        ]
        Resource = [
          aws_s3vectors_vector_bucket.vectors.vector_bucket_arn,
          aws_s3vectors_index.search_chunks.index_arn,
        ]
      }
    ]
  })
}

output "dev_aws_access_key_id" {
  value     = aws_iam_access_key.dev.id
  sensitive = true
}

output "dev_aws_secret_access_key" {
  value     = aws_iam_access_key.dev.secret
  sensitive = true
}
