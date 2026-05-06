use anyhow::Result;

use ast_call_core::confidence::ConfidenceLabel;
use ast_call_core::error::AstCallError;
use ast_call_core::index::Index;
use ast_call_core::resolve::resolve_target;
use ast_call_core::target::Target;

use crate::output::OutputOpts;

pub fn run(target_str: &str, opts: &OutputOpts) -> Result<()> {
    let target: Target = target_str
        .parse()
        .map_err(|e: String| AstCallError::ParseError(e))?;

    let index_path = crate::cmd_callers::find_index_path()?;
    let index = Index::open(&index_path)?;

    let resolved = resolve_target(&index, &target)?;
    let sym = &resolved.symbol;

    if opts.json {
        let output = serde_json::json!({
            "command": "def",
            "target": {
                "input": target_str,
            },
            "definition": {
                "name": sym.name,
                "qualified_name": sym.qualified_name,
                "kind": format!("{:?}", sym.kind).to_lowercase(),
                "file": resolved.file_path,
                "line": sym.range.start_line,
                "column": sym.range.start_col,
                "signature": sym.signature,
            },
            "confidence": 0.93,
            "confidence_label": ConfidenceLabel::from_score(0.93).as_str(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Definition:");
        println!(
            "  {}:{}:{}",
            resolved.file_path, sym.range.start_line, sym.range.start_col
        );
        if let Some(sig) = &sym.signature {
            println!("  {sig}");
        }
    }

    Ok(())
}
