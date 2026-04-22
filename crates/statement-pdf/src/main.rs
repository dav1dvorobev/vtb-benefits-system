use std::path::PathBuf;

use clap::Parser;
use statement_pdf::generate_pdf_to_file;

#[derive(Debug, Parser)]
#[command(author, version, about = "Generate a statement PDF from JSON data")]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    generate_pdf_to_file(cli.input, cli.output)?;

    Ok(())
}
