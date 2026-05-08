use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    Resolved,
    Ambiguous,
    Unresolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    pub id: i64,
    pub caller_symbol_id: i64,
    pub callee_symbol_id: Option<i64>,
    pub callee_name: Option<String>,
    pub candidate_symbol_ids: Vec<i64>,
    pub ref_id: i64,
    pub confidence: f64,
    pub resolution: Resolution,
}
