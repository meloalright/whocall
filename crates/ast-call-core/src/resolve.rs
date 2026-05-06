use crate::calls::CallEdge;
use crate::index::Index;
use crate::symbol::Symbol;
use crate::target::Target;
use anyhow::{Context, Result};

pub struct ResolvedTarget {
    pub symbol: Symbol,
    pub file_path: String,
}

pub fn resolve_target(index: &Index, target: &Target) -> Result<ResolvedTarget> {
    match target {
        Target::FileLine { file, line } => resolve_file_line(index, file, *line),
        Target::FileLineColumn { file, line, .. } => resolve_file_line(index, file, *line),
        Target::FileSymbol { file, symbol } => resolve_file_symbol(index, file, symbol),
        Target::QualifiedSymbol { path } => resolve_qualified(index, path),
        Target::PlainSymbol { name } => resolve_plain(index, name),
    }
}

fn resolve_file_line(index: &Index, file: &str, line: u32) -> Result<ResolvedTarget> {
    let file_entry = index
        .find_file(file)?
        .with_context(|| format!("file not in index: {file}"))?;

    let symbols = index.symbols_in_file(file_entry.id)?;
    let enclosing = symbols
        .into_iter()
        .filter(|s| s.contains_line(line))
        .min_by_key(|s| s.range.end_line - s.range.start_line);

    match enclosing {
        Some(symbol) => Ok(ResolvedTarget {
            file_path: file_entry.path.clone(),
            symbol,
        }),
        None => anyhow::bail!(crate::error::AstCallError::TargetOutsideSymbol),
    }
}

fn resolve_file_symbol(index: &Index, file: &str, symbol_name: &str) -> Result<ResolvedTarget> {
    let file_entry = index
        .find_file(file)?
        .with_context(|| format!("file not in index: {file}"))?;

    let symbols = index.symbols_in_file(file_entry.id)?;
    let matches: Vec<_> = symbols
        .into_iter()
        .filter(|s| s.name == symbol_name)
        .collect();

    match matches.len() {
        0 => anyhow::bail!(crate::error::AstCallError::NoMatch),
        1 => Ok(ResolvedTarget {
            file_path: file_entry.path.clone(),
            symbol: matches.into_iter().next().unwrap(),
        }),
        n => anyhow::bail!(crate::error::AstCallError::AmbiguousTarget(n)),
    }
}

fn resolve_qualified(index: &Index, path: &str) -> Result<ResolvedTarget> {
    let matches = index.find_symbols_by_qualified_name(path)?;

    match matches.len() {
        0 => anyhow::bail!(crate::error::AstCallError::NoMatch),
        1 => {
            let symbol = matches.into_iter().next().unwrap();
            let file_entry = index
                .find_file_by_id(symbol.file_id)?
                .context("file not found for symbol")?;
            Ok(ResolvedTarget {
                file_path: file_entry.path.clone(),
                symbol,
            })
        }
        n => anyhow::bail!(crate::error::AstCallError::AmbiguousTarget(n)),
    }
}

fn resolve_plain(index: &Index, name: &str) -> Result<ResolvedTarget> {
    let matches = index.find_symbols_by_name(name)?;

    match matches.len() {
        0 => anyhow::bail!(crate::error::AstCallError::NoMatch),
        1 => {
            let symbol = matches.into_iter().next().unwrap();
            let file_entry = index
                .find_file_by_id(symbol.file_id)?
                .context("file not found for symbol")?;
            Ok(ResolvedTarget {
                file_path: file_entry.path.clone(),
                symbol,
            })
        }
        n => anyhow::bail!(crate::error::AstCallError::AmbiguousTarget(n)),
    }
}

pub fn find_callers(index: &Index, target_symbol_id: i64) -> Result<Vec<CallerResult>> {
    let edges = index.call_edges_to(target_symbol_id)?;
    let mut results = Vec::new();

    for edge in edges {
        if let Some(caller_id) = Some(edge.caller_symbol_id) {
            let caller_sym = index.find_symbol_by_id(caller_id)?;
            let reference = index.find_ref_by_id(edge.ref_id)?;
            if let (Some(sym), Some(r)) = (caller_sym, reference) {
                let file = index.find_file_by_id(sym.file_id)?;
                results.push(CallerResult {
                    caller_symbol: sym,
                    call_edge: edge,
                    call_text: r.text,
                    file_path: file.map(|f| f.path).unwrap_or_default(),
                    line: r.start_line,
                    column: r.start_col,
                });
            }
        }
    }

    Ok(results)
}

pub struct CallerResult {
    pub caller_symbol: Symbol,
    pub call_edge: CallEdge,
    pub call_text: String,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
}
