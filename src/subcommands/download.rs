// TODO
// less string clones
// better path handling (using Path or PathBuf)

use anyhow::{Context, Result};
use clap::Args;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Response};
use std::cmp::min;
use std::fs::File;
use std::io::Write;

pub struct Download {
    client: Client,
}

// TODO make the path configurable as an argument
#[derive(Args)]
pub struct DownloadArgs {
    /// url of the file to download
    pub url: String,
}

impl Download {
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self { client }
    }

    fn build_progress_bar(&self, total_size: u64) -> Result<ProgressBar> {
        // Indicatif setup
        // move progress bar as a struct member field
        let pb = ProgressBar::new(total_size);
        // TODO better looking progress bar
        // see docs for more info
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{msg}\n{spinner:.red} \
            [{elapsed_precise}] \
            [{wide_bar:.cyan/blue}] \
            {bytes}/{total_bytes} \
            ({bytes_per_sec}, {eta})",
                )
                .context("Error creating progress bar template")?
                .progress_chars("||."),
        );

        Ok(pb)
    }

    async fn get_file(&self, url: &str) -> Result<Response> {
        // Reqwest setup
        self.client
            .get(url)
            .send()
            .await
            .context(format!("Failed to GET from '{}'", url))
    }

    pub async fn download_file(&self, url: String, path: String) -> Result<()> {
        let response = self.get_file(url.as_str()).await?;

        let total_size = response.content_length().context(format!(
            "Failed to get content length from '{}'",
            url.clone()
        ))?;

        // create progress bar
        let progress_bar = self.build_progress_bar(total_size)?;
        progress_bar.set_message(format!("Downloading {}", url.clone()));

        // download chunks
        let mut file = File::create(path.clone())
            .context(format!("Failed to create file '{}'", path.clone()))?;

        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            // TODO handle this properly in a match case
            let chunk = item.context("Error while downloading file".to_string())?;

            // TODO specify the chunk number in the error message
            file.write_all(&chunk)
                .context("Error while writing to file".to_string())?;

            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            progress_bar.set_position(new);
        }

        progress_bar.finish_with_message(format!("Downloaded {} to {}", url.clone(), path.clone()));

        Ok(())
    }
}
