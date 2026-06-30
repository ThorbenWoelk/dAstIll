# Remove former AWS resources from any remaining Terraform state without deleting
# source data. The runtime and active infrastructure no longer depend on them.

removed {
  from = aws_iam_openid_connect_provider.gcp

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_iam_openid_connect_provider.github

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_iam_role.backend_s3

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_iam_role.github_terraform

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_iam_role_policy.backend_polly

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_iam_role_policy.backend_s3_data

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_iam_role_policy.backend_s3_vectors

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_iam_role_policy.github_terraform

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_s3_account_public_access_block.account

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_s3_bucket.data

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_s3_bucket_lifecycle_configuration.data

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_s3_bucket_public_access_block.data

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_s3_bucket_server_side_encryption_configuration.data

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_s3_bucket_versioning.data

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_s3vectors_index.search_chunks

  lifecycle {
    destroy = false
  }
}

removed {
  from = aws_s3vectors_vector_bucket.vectors

  lifecycle {
    destroy = false
  }
}
