use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    FileLineColumn {
        file: String,
        line: u32,
        column: u32,
    },
    FileLine {
        file: String,
        line: u32,
    },
    FileSymbol {
        file: String,
        symbol: String,
    },
    QualifiedSymbol {
        path: String,
    },
    PlainSymbol {
        name: String,
    },
}

impl Target {
    pub fn file(&self) -> Option<&str> {
        match self {
            Self::FileLineColumn { file, .. }
            | Self::FileLine { file, .. }
            | Self::FileSymbol { file, .. } => Some(file),
            _ => None,
        }
    }

    pub fn line(&self) -> Option<u32> {
        match self {
            Self::FileLineColumn { line, .. } | Self::FileLine { line, .. } => Some(*line),
            _ => None,
        }
    }
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("empty target".to_string());
        }

        // file#symbol
        if let Some((file, symbol)) = s.split_once('#') {
            return Ok(Target::FileSymbol {
                file: file.to_string(),
                symbol: symbol.to_string(),
            });
        }

        // Try file:line or file:line:column
        if let Some(colon_pos) = s.rfind(':') {
            let after_last_colon = &s[colon_pos + 1..];
            if let Ok(num) = after_last_colon.parse::<u32>() {
                let before_last_colon = &s[..colon_pos];
                // Check for file:line:column
                if let Some(colon_pos2) = before_last_colon.rfind(':') {
                    let middle = &before_last_colon[colon_pos2 + 1..];
                    if let Ok(line) = middle.parse::<u32>() {
                        let file = &before_last_colon[..colon_pos2];
                        if !file.is_empty()
                            && !file.chars().all(|c| c.is_alphanumeric() || c == '_')
                        {
                            return Ok(Target::FileLineColumn {
                                file: file.to_string(),
                                line,
                                column: num,
                            });
                        }
                    }
                }
                // file:line
                if !before_last_colon.is_empty()
                    && (before_last_colon.contains('/') || before_last_colon.contains('.'))
                {
                    return Ok(Target::FileLine {
                        file: before_last_colon.to_string(),
                        line: num,
                    });
                }
            }
        }

        // Qualified symbol: contains ::
        if s.contains("::") {
            return Ok(Target::QualifiedSymbol {
                path: s.to_string(),
            });
        }

        Ok(Target::PlainSymbol {
            name: s.to_string(),
        })
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileLineColumn { file, line, column } => write!(f, "{file}:{line}:{column}"),
            Self::FileLine { file, line } => write!(f, "{file}:{line}"),
            Self::FileSymbol { file, symbol } => write!(f, "{file}#{symbol}"),
            Self::QualifiedSymbol { path } => write!(f, "{path}"),
            Self::PlainSymbol { name } => write!(f, "{name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_file_line() {
        let t: Target = "src/render.rs:42".parse().unwrap();
        assert_eq!(
            t,
            Target::FileLine {
                file: "src/render.rs".to_string(),
                line: 42
            }
        );
    }

    #[test]
    fn parse_file_line_column() {
        let t: Target = "src/app.rs:88:12".parse().unwrap();
        assert_eq!(
            t,
            Target::FileLineColumn {
                file: "src/app.rs".to_string(),
                line: 88,
                column: 12
            }
        );
    }

    #[test]
    fn parse_file_symbol() {
        let t: Target = "src/render.rs#render_text".parse().unwrap();
        assert_eq!(
            t,
            Target::FileSymbol {
                file: "src/render.rs".to_string(),
                symbol: "render_text".to_string()
            }
        );
    }

    #[test]
    fn parse_qualified() {
        let t: Target = "crate::text::render::render_text".parse().unwrap();
        assert_eq!(
            t,
            Target::QualifiedSymbol {
                path: "crate::text::render::render_text".to_string()
            }
        );
    }

    #[test]
    fn parse_plain() {
        let t: Target = "render_text".parse().unwrap();
        assert_eq!(
            t,
            Target::PlainSymbol {
                name: "render_text".to_string()
            }
        );
    }
}
