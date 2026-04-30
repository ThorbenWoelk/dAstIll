use super::{DataApiKeyValidation, YouTubeService};

#[test]
fn classifies_quota_exceeded_errors() {
    let body = r#"{
      "error": {
        "code": 403,
        "message": "The request cannot be completed because you have exceeded your quota.",
        "errors": [
          {
            "message": "The request cannot be completed because you have exceeded your quota.",
            "domain": "youtube.quota",
            "reason": "quotaExceeded"
          }
        ]
      }
    }"#;

    assert_eq!(
        YouTubeService::classify_data_api_validation_failure(body),
        DataApiKeyValidation::QuotaExceeded {
            message: Some(
                "The request cannot be completed because you have exceeded your quota.".to_string()
            ),
        }
    );
}

#[test]
fn classifies_disabled_api_errors() {
    let body = r#"{
      "error": {
        "code": 403,
        "message": "YouTube Data API v3 has not been used in project 123 before or it is disabled. Enable it by visiting the Google API Console.",
        "errors": [
          {
            "message": "YouTube Data API v3 has not been used in project 123 before or it is disabled. Enable it by visiting the Google API Console.",
            "domain": "usageLimits",
            "reason": "accessNotConfigured"
          }
        ],
        "status": "PERMISSION_DENIED"
      }
    }"#;

    assert_eq!(
        YouTubeService::classify_data_api_validation_failure(body),
        DataApiKeyValidation::ServiceDisabled {
            reason: Some("accessNotConfigured".to_string()),
            message: Some(
                "YouTube Data API v3 has not been used in project 123 before or it is disabled. Enable it by visiting the Google API Console.".to_string()
            ),
        }
    );
}

#[test]
fn classifies_other_rejections() {
    let body = r#"{
      "error": {
        "code": 400,
        "message": "API key not valid. Please pass a valid API key.",
        "errors": [
          {
            "message": "API key not valid. Please pass a valid API key.",
            "domain": "global",
            "reason": "badRequest"
          }
        ]
      }
    }"#;

    assert_eq!(
        YouTubeService::classify_data_api_validation_failure(body),
        DataApiKeyValidation::Rejected {
            reason: Some("badRequest".to_string()),
            message: Some("API key not valid. Please pass a valid API key.".to_string()),
        }
    );
}
