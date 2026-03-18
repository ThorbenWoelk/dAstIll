use aws_credential_types::provider::future::ProvideCredentials as ProvideCredentialsFuture;
use aws_credential_types::provider::{self, ProvideCredentials};
use aws_credential_types::Credentials;

#[derive(Debug)]
pub struct GcpWifCredentialProvider {
    role_arn: String,
    audience: String,
    region: String,
}

impl GcpWifCredentialProvider {
    pub fn new(role_arn: String, audience: String, region: String) -> Self {
        Self {
            role_arn,
            audience,
            region,
        }
    }

    async fn load_credentials(&self) -> provider::Result {
        let gcp_token = fetch_gcp_identity_token(&self.audience)
            .await
            .map_err(|e| {
                provider::error::CredentialsError::provider_error(format!(
                    "failed to fetch GCP identity token: {e}"
                ))
            })?;

        // STS AssumeRoleWithWebIdentity is an unauthenticated call (the web
        // identity token IS the proof), so we build a minimal STS client with
        // a dummy credential provider that always fails -- the SDK never
        // actually invokes it for this operation.
        let sts_config = aws_sdk_sts::config::Builder::new()
            .region(aws_sdk_sts::config::Region::new(self.region.clone()))
            .behavior_version_latest()
            .credentials_provider(NoCredentials)
            .build();
        let sts_client = aws_sdk_sts::Client::from_conf(sts_config);

        let response = sts_client
            .assume_role_with_web_identity()
            .role_arn(&self.role_arn)
            .role_session_name("dastill-backend")
            .web_identity_token(&gcp_token)
            .send()
            .await
            .map_err(|e| {
                provider::error::CredentialsError::provider_error(format!(
                    "STS AssumeRoleWithWebIdentity failed: {e}"
                ))
            })?;

        let sts_creds = response.credentials().ok_or_else(|| {
            provider::error::CredentialsError::provider_error(
                "STS response missing credentials".to_string(),
            )
        })?;

        let expiration = std::time::SystemTime::try_from(sts_creds.expiration().clone()).ok();

        Ok(Credentials::new(
            sts_creds.access_key_id(),
            sts_creds.secret_access_key(),
            Some(sts_creds.session_token().to_string()),
            expiration,
            "gcp-wif",
        ))
    }
}

impl ProvideCredentials for GcpWifCredentialProvider {
    fn provide_credentials<'a>(&'a self) -> ProvideCredentialsFuture<'a>
    where
        Self: 'a,
    {
        ProvideCredentialsFuture::new(self.load_credentials())
    }
}

#[derive(Debug)]
struct NoCredentials;

impl ProvideCredentials for NoCredentials {
    fn provide_credentials<'a>(&'a self) -> ProvideCredentialsFuture<'a>
    where
        Self: 'a,
    {
        ProvideCredentialsFuture::new(async {
            Err(provider::error::CredentialsError::not_loaded(
                "no credentials needed for AssumeRoleWithWebIdentity",
            ))
        })
    }
}

async fn fetch_gcp_identity_token(audience: &str) -> Result<String, String> {
    let url = format!(
        "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/identity?audience={audience}&format=full"
    );
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Metadata-Flavor", "Google")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "GCP metadata server returned {}",
            response.status()
        ));
    }

    response.text().await.map_err(|e| e.to_string())
}
