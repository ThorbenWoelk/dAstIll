#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProcessingRole {
    #[default]
    Combined,
    LocalFetcher,
    Remote,
}

impl ProcessingRole {
    pub fn from_env() -> Result<Self, String> {
        Self::parse(&std::env::var("PROCESSING_ROLE").unwrap_or_else(|_| "combined".into()))
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value.trim() {
            "combined" => Ok(Self::Combined),
            "local-fetcher" => Ok(Self::LocalFetcher),
            "remote" => Ok(Self::Remote),
            _ => Err("PROCESSING_ROLE must be combined, local-fetcher, or remote".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processing_roles_are_explicit() {
        assert_eq!(
            ProcessingRole::parse("local-fetcher"),
            Ok(ProcessingRole::LocalFetcher)
        );
        assert_eq!(ProcessingRole::parse("remote"), Ok(ProcessingRole::Remote));
        assert_eq!(
            ProcessingRole::parse("combined"),
            Ok(ProcessingRole::Combined)
        );
        assert!(ProcessingRole::parse("cloud").is_err());
    }
}
