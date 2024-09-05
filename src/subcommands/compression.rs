use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Args;
use thiserror::Error;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[derive(Debug, Error)]
enum CompressError {
    #[error("Archive name can't be more than 100 characters!")]
    ArchiveNameTooLong,
    #[error("Destination path is not found!")]
    DestinationPathNotFound,
}

pub struct Compress;

#[derive(Args)]
pub struct CompressArgs {
    pub archive_name: String,
    pub archive_dest: String,
    pub method: String,
    pub files: Vec<String>,
}

impl Compress {
    fn validate_path(path: &String) -> Result<bool> {
        let path = Path::new(path);

        match path.try_exists() {
            Ok(true) => {
                // check if the path is a file which is not correct
                // or, we could append postfix "/" and check if exists, if not, that's definitely
                // an error and we don't create shit
                todo!()
            }
            Ok(false) => Ok(false),
            Err(_e) => Err(CompressError::DestinationPathNotFound.into()),
        }
    }

    fn validate_name(archive_name: &String) -> Result<()> {
        if archive_name.len() > 100 {
            return Err(CompressError::ArchiveNameTooLong.into());
        }

        Ok(())
    }

    pub fn compress(archive_path: String, archive_name: String, files: Vec<String>) -> Result<()> {
        Self::validate_name(&archive_name)?;

        if !Self::validate_path(&archive_path)? {
            // path does not exist
            // what do you think you're doing?
        }

        let full_path_with_name = format!("{}{}", archive_path, archive_name);
        println!("archive_path: {full_path_with_name}");
        let archive_full_path = Path::new(&full_path_with_name);
        let zip_file = File::create(&archive_full_path)?;

        let mut zip_archive = ZipWriter::new(zip_file);

        let files_to_compress = files
            .into_iter()
            .map(|file| PathBuf::from(file))
            .collect::<Vec<PathBuf>>();

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        let mut buffer = vec![];

        // convert into walkdir iterator and handle nested folder compression
        for file_path in files_to_compress {
            let mut file = File::open(&file_path)?;
            let file_name = file_path.file_name().unwrap().to_str().unwrap();

            zip_archive.start_file(file_name, options)?;

            file.read_to_end(&mut buffer)
                .context("Failed to read data from file")?;

            zip_archive
                .write_all(&buffer)
                .context("Failed to write data into the zip archive")?;

            buffer.clear();
        }

        zip_archive
            .finish()
            .context("Failed to finish zip archive")?;

        Ok(())
    }
}
