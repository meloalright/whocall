use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefKind {
    Call,
    Import,
    ReExport,
    FunctionPointer,
    TraitRef,
    TypeRef,
    Test,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub id: i64,
    pub target_symbol_id: i64,
    pub source_file_id: i64,
    pub source_symbol_id: Option<i64>,
    pub kind: RefKind,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
    pub text: String,
    pub confidence: f64,
}
