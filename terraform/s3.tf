resource "aws_s3_account_public_access_block" "account" {
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket" "data" {
  bucket = "${var.app_name}-data-${var.aws_region}"

  tags = {
    App = var.app_name
  }
}

resource "aws_s3_bucket_versioning" "data" {
  bucket = aws_s3_bucket.data.id

  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "data" {
  bucket = aws_s3_bucket.data.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "data" {
  bucket = aws_s3_bucket.data.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3vectors_vector_bucket" "vectors" {
  vector_bucket_name = "${var.app_name}-vectors"
}

resource "aws_s3vectors_index" "search_chunks" {
  vector_bucket_name = aws_s3vectors_vector_bucket.vectors.vector_bucket_name
  index_name         = "search-chunks"
  data_type          = "float32"
  dimension          = 512
  distance_metric    = "cosine"

  metadata_configuration {
    non_filterable_metadata_keys = ["chunk_text", "section_title"]
  }
}

output "s3_data_bucket" {
  value = aws_s3_bucket.data.bucket
}

output "s3_vector_bucket" {
  value = aws_s3vectors_vector_bucket.vectors.vector_bucket_name
}

output "s3_vector_bucket_arn" {
  value = aws_s3vectors_vector_bucket.vectors.vector_bucket_arn
}

output "s3_vector_index_arn" {
  value = aws_s3vectors_index.search_chunks.index_arn
}
