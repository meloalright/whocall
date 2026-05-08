use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    NoMatch = 1,
    AmbiguousTarget = 2,
    ParseError = 3,
    IndexMissing = 4,
    UnsupportedLanguage = 5,
    TargetOutsideSymbol = 6,
    InternalError = 10,
}

impl ExitCode {
    pub fn code(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Error)]
pub enum WhoError {
    #[error("no match found for target")]
    NoMatch,

    #[error("ambiguous target: {0} candidates")]
    AmbiguousTarget(usize),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("index missing at {0}")]
    IndexMissing(String),

    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("target is outside any symbol")]
    TargetOutsideSymbol,

    #[error("internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

impl WhoError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::NoMatch => ExitCode::NoMatch,
            Self::AmbiguousTarget(_) => ExitCode::AmbiguousTarget,
            Self::ParseError(_) => ExitCode::ParseError,
            Self::IndexMissing(_) => ExitCode::IndexMissing,
            Self::UnsupportedLanguage(_) => ExitCode::UnsupportedLanguage,
            Self::TargetOutsideSymbol => ExitCode::TargetOutsideSymbol,
            Self::Internal(_) | Self::Io(_) | Self::Sqlite(_) => ExitCode::InternalError,
        }
    }
}
