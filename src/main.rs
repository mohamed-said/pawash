use std::cmp::min;
use std::fs::File;
use std::io::Write;

use anyhow::{Context, Result};
use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;

// TODO
// less string clones
// better path handling (using Path or PathBuf)
pub async fn download_file(client: &Client, url: String, path: String) -> Result<()> {
    // Reqwest setup
    let res = client
        .get(url.clone())
        .send()
        .await
        .context(format!("Failed to GET from '{}'", url.clone()))?;

    let total_size = res
        .content_length()
        .context(format!("Failed to get content length from '{}'", url.clone()))?;

    // Indicatif setup
    // move progress bar as a struct member field
    let pb = ProgressBar::new(total_size);
    // TODO better looking progress bar
    // see docs for more info
    pb.set_style(
        ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.red} \
            [{elapsed_precise}] \
            [{wide_bar:.cyan/blue}] \
            {bytes}/{total_bytes} \
            ({bytes_per_sec}, {eta})"
        )
        .context("Error creating progress bar template")?
        .progress_chars("||."));

    pb.set_message(format!("Downloading {}", url.clone()));

    // download chunks
    let mut file = File::create(path.clone())
        .context(format!("Failed to create file '{}'", path.clone()))?;

    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        // TODO handle this properly in a match case
        let chunk = item.context(format!("Error while downloading file"))?;

        // TODO specify the chunk number in the error message
        file
            .write_all(&chunk)
            .context(format!("Error while writing to file"))?;

        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {} to {}", url.clone(), path.clone()));
    return Ok(());
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::new();

    download_file(&client, "https://ash-speed.hetzner.com/1GB.bin".to_string(), "new_file".to_string()).await?;

    Ok(())
}
