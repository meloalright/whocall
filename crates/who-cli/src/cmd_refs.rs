use anyhow::Result;

use who_core::error::WhoError;
use who_core::index::Index;
use who_core::refs::RefKind;
use who_core::resolve::resolve_target;
use who_core::target::Target;

use crate::output::OutputOpts;

pub fn run(target_str: &str, opts: &OutputOpts) -> Result<()> {
    let target: Target = target_str
        .parse()
        .map_err(|e: String| WhoError::ParseError(e))?;

    let index_path = crate::cmd_callers::find_index_path()?;
    let index = Index::open(&index_path)?;

    let resolved = resolve_target(&index, &target)?;
    let refs = index.refs_to_symbol(resolved.symbol.id)?;

    if opts.json {
        let output = serde_json::json!({
            "command": "refs",
            "target": {
                "input": target_str,
                "symbol": resolved.symbol.name,
                "qualified_name": resolved.symbol.qualified_name,
            },
            "references": refs.iter().map(|r| {
                let file = index.find_file_by_id(r.source_file_id).ok().flatten();
                serde_json::json!({
                    "kind": format!("{:?}", r.kind).to_lowercase(),
                    "file": file.as_ref().map(|f| f.path.as_str()).unwrap_or("?"),
                    "line": r.start_line,
                    "column": r.start_col,
                    "text": r.text,
                    "confidence": r.confidence,
                })
            }).collect::<Vec<_>>(),
            "total": refs.len(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if opts.ndjson {
        for r in &refs {
            let file = index.find_file_by_id(r.source_file_id).ok().flatten();
            let line = serde_json::json!({
                "kind": format!("{:?}", r.kind).to_lowercase(),
                "file": file.as_ref().map(|f| f.path.as_str()).unwrap_or("?"),
                "line": r.start_line,
                "column": r.start_col,
                "text": r.text,
            });
            println!("{}", serde_json::to_string(&line)?);
        }
    } else {
        println!("References to {}:", resolved.symbol.name);
        println!();

        let imports: Vec<_> = refs.iter().filter(|r| r.kind == RefKind::Import).collect();
        let calls: Vec<_> = refs.iter().filter(|r| r.kind == RefKind::Call).collect();
        let reexports: Vec<_> = refs
            .iter()
            .filter(|r| r.kind == RefKind::ReExport)
            .collect();
        let others: Vec<_> = refs
            .iter()
            .filter(|r| !matches!(r.kind, RefKind::Import | RefKind::Call | RefKind::ReExport))
            .collect();

        if !imports.is_empty() {
            println!("Imports:");
            for r in &imports {
                let file = index.find_file_by_id(r.source_file_id).ok().flatten();
                println!(
                    "  {}:{}:{}\t{}",
                    file.as_ref().map(|f| f.path.as_str()).unwrap_or("?"),
                    r.start_line,
                    r.start_col,
                    r.text
                );
            }
            println!();
        }

        if !calls.is_empty() {
            println!("Calls:");
            for r in &calls {
                let file = index.find_file_by_id(r.source_file_id).ok().flatten();
                println!(
                    "  {}:{}:{}\t{}",
                    file.as_ref().map(|f| f.path.as_str()).unwrap_or("?"),
                    r.start_line,
                    r.start_col,
                    r.text
                );
            }
            println!();
        }

        if !reexports.is_empty() {
            println!("Re-exports:");
            for r in &reexports {
                let file = index.find_file_by_id(r.source_file_id).ok().flatten();
                println!(
                    "  {}:{}:{}\t{}",
                    file.as_ref().map(|f| f.path.as_str()).unwrap_or("?"),
                    r.start_line,
                    r.start_col,
                    r.text
                );
            }
            println!();
        }

        if !others.is_empty() {
            println!("Other:");
            for r in &others {
                let file = index.find_file_by_id(r.source_file_id).ok().flatten();
                println!(
                    "  {}:{}:{}\t{:?} {}",
                    file.as_ref().map(|f| f.path.as_str()).unwrap_or("?"),
                    r.start_line,
                    r.start_col,
                    r.kind,
                    r.text
                );
            }
            println!();
        }

        println!("{} references found.", refs.len());
    }

    Ok(())
}
