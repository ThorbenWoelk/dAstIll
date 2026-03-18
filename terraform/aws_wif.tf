data "google_project" "current" {}

resource "aws_iam_openid_connect_provider" "gcp" {
  url             = "https://accounts.google.com"
  client_id_list  = [google_service_account.backend_sa.unique_id]
  thumbprint_list = ["08745487e891c19e3078c1f2a07e452950ef36f6"]
}

resource "aws_iam_role" "backend_s3" {
  name = "${var.app_name}-gcp-backend"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Principal = {
          Federated = aws_iam_openid_connect_provider.gcp.arn
        }
        Action = "sts:AssumeRoleWithWebIdentity"
        Condition = {
          StringEquals = {
            "accounts.google.com:sub" = google_service_account.backend_sa.unique_id
            "accounts.google.com:aud" = google_service_account.backend_sa.unique_id
          }
        }
      }
    ]
  })
}

resource "aws_iam_role_policy" "backend_s3_data" {
  name = "${var.app_name}-s3-data"
  role = aws_iam_role.backend_s3.id

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

resource "aws_iam_role_policy" "backend_s3_vectors" {
  name = "${var.app_name}-s3-vectors"
  role = aws_iam_role.backend_s3.id

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

output "aws_backend_role_arn" {
  value = aws_iam_role.backend_s3.arn
}
