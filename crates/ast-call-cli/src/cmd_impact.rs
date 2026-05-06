use std::collections::HashSet;

use anyhow::Result;

use ast_call_core::error::AstCallError;
use ast_call_core::index::Index;
use ast_call_core::resolve::{find_callers, resolve_target};
use ast_call_core::symbol::Symbol;
use ast_call_core::target::Target;

use crate::output::OutputOpts;

pub fn run(target_str: &str, depth: u32, _tests: bool, opts: &OutputOpts) -> Result<()> {
    let target: Target = target_str
        .parse()
        .map_err(|e: String| AstCallError::ParseError(e))?;

    let index_path = crate::cmd_callers::find_index_path()?;
    let index = Index::open(&index_path)?;

    let resolved = resolve_target(&index, &target)?;

    let mut layers: Vec<Vec<(Symbol, String)>> = Vec::new();
    let mut seen = HashSet::new();
    seen.insert(resolved.symbol.id);

    let mut current_ids = vec![resolved.symbol.id];

    for _ in 0..depth {
        let mut layer = Vec::new();
        let mut next_ids = Vec::new();

        for id in &current_ids {
            let callers = find_callers(&index, *id)?;
            for c in callers {
                if seen.insert(c.caller_symbol.id) {
                    next_ids.push(c.caller_symbol.id);
                    layer.push((c.caller_symbol, c.file_path));
                }
            }
        }

        if layer.is_empty() {
            break;
        }
        layers.push(layer);
        current_ids = next_ids;
    }

    if opts.json {
        let output = serde_json::json!({
            "command": "impact",
            "target": {
                "input": target_str,
                "symbol": resolved.symbol.name,
                "qualified_name": resolved.symbol.qualified_name,
            },
            "layers": layers.iter().enumerate().map(|(i, layer)| {
                serde_json::json!({
                    "depth": i + 1,
                    "callers": layer.iter().map(|(s, fp)| {
                        serde_json::json!({
                            "symbol": s.name,
                            "qualified_name": s.qualified_name,
                            "file": fp,
                            "line": s.range.start_line,
                        })
                    }).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Changing {} may affect:", resolved.symbol.name);
        println!();

        for (i, layer) in layers.iter().enumerate() {
            if i == 0 {
                println!("Direct callers:");
            } else {
                println!("Caller chain depth {}:", i + 1);
            }
            for (sym, file_path) in layer {
                println!("  {}:{}\t{}", file_path, sym.range.start_line, sym.name);
            }
            println!();
        }

        let total: usize = layers.iter().map(|l| l.len()).sum();
        println!("{total} affected symbols found.");
    }

    Ok(())
}
