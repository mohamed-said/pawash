pub mod subcommands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use subcommands::download::Download;

/// A simple command-line tool with subcommands
#[derive(Parser)]
#[command(
    name = "pawash",
    version = "0.1",
    author = "Mohamed Said mohamed.said.fci@gmail.com",
    about = "A command line helper tool. Cooler than you think!"
)]
struct Pawash {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Downloads a file from given url
    Download { source: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let pawash = Pawash::parse();

    match &pawash.command {
        Commands::Download { source } => {
            let download = Download::new();
            download
                .download_file(source.to_owned(), String::from("new_file"))
                .await?;
            println!("hoppa");
        }
    }

    Ok(())
}
