use std::{
    borrow::Cow,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Args;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[derive(Debug, Error)]
enum CompressError {
    #[error("Archive name can't be more than 100 characters!")]
    ArchiveNameTooLong,
    #[error("Destination path is not found!")]
    DestinationPathNotFound,
    #[error("Destination is not a directory!")]
    NotADirectory,
}

pub struct Compress;

#[derive(Args)]
pub struct CompressArgs {
    pub archive_name: String,
    pub archive_dest: String,
    // TODO enable an argument and support defaults
    //pub method: String,
    pub src_dir: String,
}

impl Compress {
    pub fn compress(archive_path: &str, archive_name: &str, src_dir: &str) -> Result<()> {
        let archive_name = Self::validate_name(archive_name)?;

        // validate src_dir path
        let src_dir = Self::validate_path(src_dir)?;

        // validate destination path for archive
        let mut archive_path = Self::validate_path(archive_path)?;
        archive_path.push(archive_name.into_owned());

        let src_dir = Path::new(&src_dir);

        // TODO consider making this an argument
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        let dir_to_compress = Path::new(&src_dir);
        let walk_dir = WalkDir::new(dir_to_compress);
        let mut entry_iter = walk_dir.into_iter().filter_map(|entry| entry.ok());

        Self::do_compress(
            &mut entry_iter,
            dir_to_compress,
            archive_path.as_path(),
            options,
        )?;

        Ok(())
    }

    fn do_compress(
        it: &mut dyn Iterator<Item = DirEntry>,
        src_dir: &Path,
        dst_file: &Path,
        options: SimpleFileOptions,
    ) -> Result<()> {
        let zip_file = File::create(dst_file)?;
        let mut zip_archive = ZipWriter::new(zip_file);

        let mut buffer = vec![];
        for entry in it {
            let path = entry.path();
            let name = path.strip_prefix(src_dir)?;
            let path_string = name
                .to_str()
                .map(str::to_owned)
                .with_context(|| format!("{name:?} Is a Non UTF-8 Path"))?;

            if path.is_file() {
                println!("adding file {path:?} as {name:?} ...");
                zip_archive.start_file(path_string, options)?;
                let mut file = File::open(path)?;

                file.read_to_end(&mut buffer)?;
                zip_archive.write_all(&buffer)?;
                buffer.clear();
            } else if !name.as_os_str().is_empty() {
                // Only if not root! Avoids path spec / warning
                // and mapname conversion failed error on unzip
                println!("adding dir {path_string:?} as {name:?} ...");
                zip_archive.add_directory(path_string, options)?;
            }
        }

        zip_archive.finish()?;
        Ok(())
    }

    fn validate_path(dir: &str) -> Result<PathBuf> {
        let mut path = PathBuf::from(dir);
        if let Ok(canonical_path) = std::fs::canonicalize(&path) {
            path = canonical_path;
            if !path.is_dir() {
                return Err(CompressError::NotADirectory.into());
            }
        } else {
            return Err(CompressError::DestinationPathNotFound.into());
        }

        Ok(path)
    }

    fn validate_name(archive_name: &str) -> Result<Cow<'_, str>> {
        if archive_name.len() > 100 {
            return Err(CompressError::ArchiveNameTooLong.into());
        }

        if !archive_name.ends_with(".zip") {
            let mut name = String::with_capacity(archive_name.len() + 4);
            name.push_str(archive_name);
            name.push_str(".zip");
            return Ok(Cow::Owned(name));
        }

        Ok(Cow::Borrowed(archive_name))
    }
}
