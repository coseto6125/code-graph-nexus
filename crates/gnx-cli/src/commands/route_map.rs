use crate::engine::Engine;
use crate::output::{emit, OutputFormat};
use clap::Args;
use gnx_core::graph::ArchivedNodeKind;
use gnx_core::GnxError;

#[derive(Args, Debug)]
pub struct RouteMapArgs {
    #[arg(long)]
    pub repo: Option<String>,

    /// Output format
    #[arg(long, default_value = "toon")]
    pub format: Option<String>,
}

pub fn run(args: RouteMapArgs, engine: &Engine) -> Result<(), GnxError> {
    let graph = engine.graph().map_err(|e| GnxError::Rkyv(e.to_string()))?;
    let format = OutputFormat::parse(args.format.as_deref());

    let mut results = Vec::new();

    for node in graph.nodes.iter() {
        if matches!(&node.kind, ArchivedNodeKind::Route) {
            let name = node.name.resolve(&graph.string_pool);
            let file_node = &graph.files[node.file_idx.to_native() as usize];
            results.push(serde_json::json!({
                "uid": node.uid.resolve(&graph.string_pool),
                "name": name,
                "kind": "Route",
                "filePath": file_node.path.resolve(&graph.string_pool),
                "line": node.span.0.to_native(),
            }));
        }
    }

    let result = serde_json::json!({
        "status": "success",
        "results": results,
    });

    emit(&result, format)
}
