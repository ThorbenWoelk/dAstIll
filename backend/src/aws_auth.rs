use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_credential_types::Credentials;
use aws_credential_types::provider::future::ProvideCredentials as ProvideCredentialsFuture;
use aws_credential_types::provider::{self, ProvideCredentials};

#[derive(Debug, Clone)]
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

        let expiration = std::time::SystemTime::try_from(*sts_creds.expiration()).ok();

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AwsCredentialMode {
    DefaultChain,
    GcpWif { role_arn: String, audience: String },
}

pub fn credential_mode_from_env() -> Result<AwsCredentialMode, String> {
    credential_mode_from_values(
        std::env::var("AWS_ROLE_ARN").ok(),
        std::env::var("AWS_WIF_AUDIENCE").ok(),
    )
}

pub fn credential_mode_from_values(
    role_arn: Option<String>,
    audience: Option<String>,
) -> Result<AwsCredentialMode, String> {
    let role_arn = normalize_env_value(role_arn);
    let audience = normalize_env_value(audience);

    match (role_arn, audience) {
        (Some(role_arn), Some(audience)) => Ok(AwsCredentialMode::GcpWif { role_arn, audience }),
        (None, None) => Ok(AwsCredentialMode::DefaultChain),
        (Some(_), None) => Err(
            "AWS_ROLE_ARN is set but AWS_WIF_AUDIENCE is missing; both must be set for GCP AWS WIF"
                .to_string(),
        ),
        (None, Some(_)) => Err(
            "AWS_WIF_AUDIENCE is set but AWS_ROLE_ARN is missing; both must be set for GCP AWS WIF"
                .to_string(),
        ),
    }
}

pub async fn load_aws_sdk_config(region: String) -> Result<SdkConfig, String> {
    let loader =
        aws_config::defaults(BehaviorVersion::latest()).region(Region::new(region.clone()));

    match credential_mode_from_env()? {
        AwsCredentialMode::DefaultChain => Ok(loader.load().await),
        AwsCredentialMode::GcpWif { role_arn, audience } => {
            tracing::info!("using GCP AWS WIF credential provider");
            Ok(loader
                .credentials_provider(GcpWifCredentialProvider::new(role_arn, audience, region))
                .load()
                .await)
        }
    }
}

fn normalize_env_value(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
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

#[cfg(test)]
mod tests {
    use super::{AwsCredentialMode, credential_mode_from_values};

    #[test]
    fn credential_mode_defaults_to_sdk_chain_when_wif_values_are_absent() {
        assert_eq!(
            credential_mode_from_values(None, None).expect("mode"),
            AwsCredentialMode::DefaultChain
        );
    }

    #[test]
    fn credential_mode_uses_gcp_wif_when_both_values_are_present() {
        assert_eq!(
            credential_mode_from_values(
                Some(" arn:aws:iam::123456789012:role/test ".to_string()),
                Some(" audience-value ".to_string())
            )
            .expect("mode"),
            AwsCredentialMode::GcpWif {
                role_arn: "arn:aws:iam::123456789012:role/test".to_string(),
                audience: "audience-value".to_string(),
            }
        );
    }

    #[test]
    fn credential_mode_rejects_partial_wif_configuration() {
        let err = credential_mode_from_values(
            Some("arn:aws:iam::123456789012:role/test".to_string()),
            None,
        )
        .expect_err("partial config should fail");
        assert!(err.contains("AWS_WIF_AUDIENCE"));

        let err = credential_mode_from_values(None, Some("audience-value".to_string()))
            .expect_err("partial config should fail");
        assert!(err.contains("AWS_ROLE_ARN"));
    }
}
