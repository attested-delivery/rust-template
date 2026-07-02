//! Command-line interface for the MIF (Modeled Information Format) ecosystem.
//!
//! A CLI naturally writes to stdout/stderr; this binary exempts itself from
//! the workspace's `print_stdout`/`print_stderr` lints for that reason (see
//! this repo's `CLAUDE.md`, "Lint Configuration").
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "mif-cli",
    version,
    about = "CLI for the MIF (Modeled Information Format) ecosystem"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a MIF document against the canonical schema.
    Validate {
        /// Path to the MIF document (JSON-LD projection) to validate.
        file: PathBuf,
    },
    /// Ontology-related operations.
    Ontology {
        #[command(subcommand)]
        command: OntologyCommand,
    },
}

#[derive(Subcommand)]
enum OntologyCommand {
    /// Resolve an ontology's three-tier `extends` chain.
    Resolve {
        /// The ontology ID to resolve.
        id: String,
        /// Directory containing ontology definition YAML files.
        #[arg(long)]
        ontologies_dir: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Error: {message}");
            ExitCode::FAILURE
        },
    }
}

fn run(command: &Command) -> Result<(), String> {
    match command {
        Command::Validate { file } => validate(file),
        Command::Ontology { command } => match command {
            OntologyCommand::Resolve { id, ontologies_dir } => resolve(id, ontologies_dir),
        },
    }
}

fn validate(file: &Path) -> Result<(), String> {
    let contents = std::fs::read_to_string(file)
        .map_err(|source| format!("failed to read {}: {source}", file.display()))?;
    let instance: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|source| format!("failed to parse {} as JSON: {source}", file.display()))?;
    match mif_schema::validate_document(&instance) {
        Ok(()) => {
            println!("{}: valid", file.display());
            Ok(())
        },
        Err(error) => {
            println!("{}: invalid", file.display());
            for message in error.messages() {
                println!("  - {message}");
            }
            Err(format!("{} failed schema validation", file.display()))
        },
    }
}

fn resolve(id: &str, ontologies_dir: &Path) -> Result<(), String> {
    let corpus = mif_ontology::load_corpus_from_dir(ontologies_dir).map_err(|e| e.to_string())?;
    let chain = mif_ontology::resolve_chain(id, &corpus).map_err(|e| e.to_string())?;
    for ontology in &chain {
        println!("{} ({})", ontology.id, ontology.version);
    }
    Ok(())
}
