//! MCP server for the MIF (Modeled Information Format) ecosystem.
//!
//! Exposes the same two operations as `mif-cli`, as MCP tools:
//! `validate_mif_document` and `resolve_ontology_reference`. Both are thin
//! wrappers calling the identical `mif-schema`/`mif-ontology` functions
//! `mif-cli` calls — kept deliberately in lockstep rather than diverging.

use std::path::PathBuf;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::transport::stdio;
use rmcp::{ServerHandler, ServiceExt, schemars, tool, tool_handler, tool_router};

/// Parameters for the `validate_mif_document` tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ValidateParams {
    /// Path to the MIF document (JSON-LD projection) to validate.
    file: PathBuf,
}

/// Parameters for the `resolve_ontology_reference` tool.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ResolveParams {
    /// The ontology ID to resolve.
    id: String,
    /// Directory containing ontology definition YAML files.
    ontologies_dir: PathBuf,
}

#[derive(Clone)]
struct Mif;

// rmcp's #[tool] macro requires an instance method (&self receiver) for its
// dispatch mechanism, even though these handlers are stateless.
#[allow(clippy::unused_self)]
#[tool_router]
impl Mif {
    #[tool(description = "Validate a MIF document against the canonical MIF JSON Schema")]
    fn validate_mif_document(
        &self,
        Parameters(ValidateParams { file }): Parameters<ValidateParams>,
    ) -> String {
        let contents = match std::fs::read_to_string(&file) {
            Ok(contents) => contents,
            Err(source) => return format!("failed to read {}: {source}", file.display()),
        };
        let instance: serde_json::Value = match serde_json::from_str(&contents) {
            Ok(instance) => instance,
            Err(source) => return format!("failed to parse {} as JSON: {source}", file.display()),
        };
        match mif_schema::validate_document(&instance) {
            Ok(()) => format!("{}: valid", file.display()),
            Err(error) => {
                let messages = error.messages().join("; ");
                format!("{}: invalid ({messages})", file.display())
            },
        }
    }

    #[tool(description = "Resolve an ontology's three-tier extends chain")]
    fn resolve_ontology_reference(
        &self,
        Parameters(ResolveParams { id, ontologies_dir }): Parameters<ResolveParams>,
    ) -> String {
        let corpus = match mif_ontology::load_corpus_from_dir(&ontologies_dir) {
            Ok(corpus) => corpus,
            Err(error) => return error.to_string(),
        };
        match mif_ontology::resolve_chain(&id, &corpus) {
            Ok(chain) => chain
                .iter()
                .map(|ontology| format!("{} ({})", ontology.id, ontology.version))
                .collect::<Vec<_>>()
                .join(" -> "),
            Err(error) => error.to_string(),
        }
    }
}

#[tool_handler(
    name = "mif-mcp",
    version = "0.1.0",
    instructions = "Validate MIF documents and resolve MIF ontology references"
)]
impl ServerHandler for Mif {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = Mif.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
