data "google_project" "current" {}

resource "aws_iam_openid_connect_provider" "gcp" {
  url             = "https://accounts.google.com"
  client_id_list  = [google_service_account.backend_sa.unique_id]
  thumbprint_list = ["08745487e891c19e3078c1f2a07e452950ef36f6"]
}

resource "aws_iam_openid_connect_provider" "github" {
  url             = "https://token.actions.githubusercontent.com"
  client_id_list  = ["sts.amazonaws.com"]
  thumbprint_list = ["6938fd4d98bab03faadb97b34396831e3780aea1"]
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

resource "aws_iam_role" "github_terraform" {
  name = "${var.app_name}-github-terraform"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
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
            "token.actions.githubusercontent.com:sub" = "repo:${var.github_repository}:*"
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
          "s3:GetBucketLocation",
          "s3:ListAllMyBuckets",
          "sts:GetCallerIdentity",
        ]
        Resource = [
          aws_s3_bucket.data.arn,
          "${aws_s3_bucket.data.arn}/*",
          "arn:aws:s3:::*",
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
          "${aws_s3vectors_vector_bucket.vectors.vector_bucket_arn}/*",
          aws_s3vectors_index.search_chunks.index_arn,
        ]
      }
    ]
  })
}

resource "aws_iam_role_policy" "backend_polly" {
  name = "${var.app_name}-polly"
  role = aws_iam_role.backend_s3.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "polly:SynthesizeSpeech",
          "polly:DescribeVoices",
        ]
        Resource = "*"
      }
    ]
  })
}

resource "aws_iam_role_policy" "github_terraform" {
  name = "${var.app_name}-github-terraform"
  role = aws_iam_role.github_terraform.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "iam:AddClientIDToOpenIDConnectProvider",
          "iam:CreateOpenIDConnectProvider",
          "iam:CreateRole",
          "iam:DeleteOpenIDConnectProvider",
          "iam:DeleteRole",
          "iam:DeleteRolePolicy",
          "iam:GetOpenIDConnectProvider",
          "iam:GetRole",
          "iam:GetRolePolicy",
          "iam:ListOpenIDConnectProviders",
          "iam:ListRolePolicies",
          "iam:ListRoles",
          "iam:PutRolePolicy",
          "iam:RemoveClientIDFromOpenIDConnectProvider",
          "iam:TagOpenIDConnectProvider",
          "iam:TagRole",
          "iam:UntagOpenIDConnectProvider",
          "iam:UntagRole",
          "iam:UpdateAssumeRolePolicy",
          "iam:UpdateOpenIDConnectProviderThumbprint"
        ]
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = [
          "s3:*",
          "s3control:GetPublicAccessBlock",
          "s3control:PutPublicAccessBlock",
          "s3control:DeletePublicAccessBlock",
          "s3vectors:*",
          "sts:GetCallerIdentity"
        ]
        Resource = "*"
      }
    ]
  })
}

output "aws_backend_role_arn" {
  value = aws_iam_role.backend_s3.arn
}

output "aws_github_terraform_role_arn" {
  value = aws_iam_role.github_terraform.arn
}
