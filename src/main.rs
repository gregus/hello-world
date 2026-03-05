/// mermaid-to-ppt — Convert Mermaid diagrams to PowerPoint presentations.
///
/// Currently supported diagram types:
///   • Gantt charts
///
/// Usage:
///   mermaid-to-ppt <input.mmd> [-o output.pptx]

mod gantt;
mod pptx_gantt;

use anyhow::{bail, Result};
use clap::Parser;
use mermaid_rs_renderer::DiagramKind;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "mermaid-to-ppt",
    version,
    about = "Convert Mermaid .mmd diagrams to PowerPoint .pptx files",
    long_about = None,
)]
struct Cli {
    /// Input Mermaid file (.mmd)
    input: PathBuf,

    /// Output PowerPoint file (.pptx); defaults to <input>.pptx
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let source = fs::read_to_string(&cli.input)
        .map_err(|e| anyhow::anyhow!("Cannot read {:?}: {}", cli.input, e))?;

    let output = cli
        .output
        .unwrap_or_else(|| cli.input.with_extension("pptx"));

    // 1. Parse the Mermaid source.
    let parsed = mermaid_rs_renderer::parse_mermaid(&source)
        .map_err(|e| anyhow::anyhow!("Mermaid parse error: {}", e))?;

    // 2. Dispatch by diagram kind.
    match parsed.graph.kind {
        DiagramKind::Gantt => {
            let gantt_data = gantt::GanttData::from_graph(&parsed.graph)?;
            pptx_gantt::save_gantt_pptx(&gantt_data, &output)?;
        }
        other => {
            bail!(
                "Diagram kind {:?} is not yet supported. Only Gantt is implemented.",
                other
            );
        }
    }

    println!("Saved: {}", output.display());
    Ok(())
}
