use serde::Serialize;
use who_core::resolve::CallerResult;
use who_core::symbol::Symbol;

pub struct OutputOpts {
    pub json: bool,
}

#[derive(Serialize)]
pub struct CallersOutput {
    pub command: String,
    pub target: TargetOutput,
    pub callers: Vec<CallerOutput>,
    pub summary: SummaryOutput,
}

#[derive(Serialize)]
pub struct TargetOutput {
    pub input: String,
    pub resolved_symbol: ResolvedSymbolOutput,
}

#[derive(Serialize)]
pub struct ResolvedSymbolOutput {
    pub name: String,
    pub qualified_name: String,
    pub kind: String,
    pub file: String,
    pub range: RangeOutput,
    pub signature: Option<String>,
}

#[derive(Serialize)]
pub struct RangeOutput {
    pub start: PositionOutput,
    pub end: PositionOutput,
}

#[derive(Serialize)]
pub struct PositionOutput {
    pub line: u32,
    pub column: u32,
}

#[derive(Serialize)]
pub struct CallerOutput {
    pub caller_symbol: String,
    pub caller_kind: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub call_expr: String,
}

#[derive(Serialize)]
pub struct SummaryOutput {
    pub caller_count: usize,
}

pub fn format_callers_human(
    _target_input: &str,
    symbol: &Symbol,
    file_path: &str,
    callers: &[CallerResult],
) {
    println!("Target:");
    println!("  {}", symbol.name);
    println!(
        "  {}:{}:{}",
        file_path, symbol.range.start_line, symbol.range.start_col
    );
    if let Some(sig) = &symbol.signature {
        println!("  {sig}");
    }
    println!();

    if callers.is_empty() {
        println!("No callers found.");
    } else {
        println!("Callers:");
        for c in callers {
            println!(
                "  {}:{}:{}\t{}",
                c.file_path, c.line, c.column, c.caller_symbol.name
            );
        }
        println!();
        println!("{} callers found.", callers.len());
    }
}

pub fn format_callers_json(
    target_input: &str,
    symbol: &Symbol,
    file_path: &str,
    callers: &[CallerResult],
) {
    let output = CallersOutput {
        command: "callers".to_string(),
        target: TargetOutput {
            input: target_input.to_string(),
            resolved_symbol: ResolvedSymbolOutput {
                name: symbol.name.clone(),
                qualified_name: symbol.qualified_name.clone(),
                kind: format!("{:?}", symbol.kind).to_lowercase(),
                file: file_path.to_string(),
                range: RangeOutput {
                    start: PositionOutput {
                        line: symbol.range.start_line,
                        column: symbol.range.start_col,
                    },
                    end: PositionOutput {
                        line: symbol.range.end_line,
                        column: symbol.range.end_col,
                    },
                },
                signature: symbol.signature.clone(),
            },
        },
        callers: callers
            .iter()
            .map(|c| CallerOutput {
                caller_symbol: c.caller_symbol.name.clone(),
                caller_kind: format!("{:?}", c.caller_symbol.kind).to_lowercase(),
                file: c.file_path.clone(),
                line: c.line,
                column: c.column,
                call_expr: c.call_text.clone(),
            })
            .collect(),
        summary: SummaryOutput {
            caller_count: callers.len(),
        },
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
