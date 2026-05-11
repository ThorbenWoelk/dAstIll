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
