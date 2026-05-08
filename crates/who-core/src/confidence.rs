use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLabel {
    High,
    Medium,
    Low,
    Unresolved,
}

impl ConfidenceLabel {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.85 {
            Self::High
        } else if score >= 0.60 {
            Self::Medium
        } else if score >= 0.30 {
            Self::Low
        } else {
            Self::Unresolved
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Unresolved => "unresolved",
        }
    }
}

pub struct ConfidenceBuilder {
    score: f64,
}

impl ConfidenceBuilder {
    pub fn new() -> Self {
        Self { score: 0.0 }
    }

    pub fn exact_source_location(mut self) -> Self {
        self.score += 0.40;
        self
    }

    pub fn exact_qualified_match(mut self) -> Self {
        self.score += 0.35;
        self
    }

    pub fn import_resolves(mut self) -> Self {
        self.score += 0.25;
        self
    }

    pub fn receiver_type_known(mut self) -> Self {
        self.score += 0.25;
        self
    }

    pub fn same_module(mut self) -> Self {
        self.score += 0.10;
        self
    }

    pub fn exact_local_name(mut self) -> Self {
        self.score += 0.10;
        self
    }

    pub fn multiple_candidates(mut self) -> Self {
        self.score -= 0.20;
        self
    }

    pub fn dynamic_dispatch(mut self) -> Self {
        self.score -= 0.25;
        self
    }

    pub fn macro_generated(mut self) -> Self {
        self.score -= 0.30;
        self
    }

    pub fn dynamic_import(mut self) -> Self {
        self.score -= 0.40;
        self
    }

    pub fn build(self) -> f64 {
        self.score.clamp(0.0, 1.0)
    }

    pub fn label(&self) -> ConfidenceLabel {
        ConfidenceLabel::from_score(self.score.clamp(0.0, 1.0))
    }
}

impl Default for ConfidenceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
