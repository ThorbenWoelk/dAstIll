use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatQueryIntent {
    Fact,
    Synthesis,
    Pattern,
    Comparison,
    RecentActivity,
}

impl ChatQueryIntent {
    pub(super) fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "fact" => Some(Self::Fact),
            "synthesis" => Some(Self::Synthesis),
            "pattern" => Some(Self::Pattern),
            "comparison" => Some(Self::Comparison),
            "recent_activity" | "recent-activity" | "recent" => Some(Self::RecentActivity),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Fact => "fact lookup",
            Self::Synthesis => "targeted synthesis",
            Self::Pattern => "broad pattern analysis",
            Self::Comparison => "comparison",
            Self::RecentActivity => "recent activity",
        }
    }

    pub(super) fn needs_synthesis_stage(&self) -> bool {
        matches!(
            self,
            Self::Pattern | Self::Comparison | Self::RecentActivity
        )
    }
}
