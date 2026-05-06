pub(super) struct AwsClients {
    pub(super) config: aws_config::SdkConfig,
    pub(super) s3: aws_sdk_s3::Client,
    pub(super) s3v: aws_sdk_s3vectors::Client,
}

pub(super) async fn build_aws_clients(aws_region: &str) -> anyhow::Result<AwsClients> {
    let config = crate::aws_auth::load_aws_sdk_config(aws_region.to_string())
        .await
        .map_err(|err| anyhow::anyhow!(err))?;

    let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&config);
    if let Ok(endpoint) = std::env::var("S3_ENDPOINT_URL") {
        tracing::info!(endpoint = %endpoint, "using custom S3 endpoint");
        s3_config_builder = s3_config_builder
            .endpoint_url(endpoint)
            .force_path_style(true);
    }
    let s3 = aws_sdk_s3::Client::from_conf(s3_config_builder.build());

    let mut s3v_config_builder = aws_sdk_s3vectors::config::Builder::from(&config);
    if let Ok(endpoint) = std::env::var("S3_VECTOR_ENDPOINT_URL") {
        tracing::info!(endpoint = %endpoint, "using custom S3 Vectors endpoint");
        s3v_config_builder = s3v_config_builder.endpoint_url(endpoint);
    }
    let s3v = aws_sdk_s3vectors::Client::from_conf(s3v_config_builder.build());

    Ok(AwsClients { config, s3, s3v })
}
