use std::{
    fs::{DirEntry, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Args;
use thiserror::Error;
use walkdir::WalkDir;
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
    pub src_dir: String,
}

impl Compress {
    fn validate_path(dir: &String) -> Result<bool> {
        let mut dir = PathBuf::from(dir);
        if !dir.ends_with("/") {
            dir.push("/");
        }
        let path = Path::new(&dir);

        match path.try_exists() {
            Ok(true) => Ok(true),
            Ok(false) => Err(CompressError::DestinationPathNotFound.into()),
            Err(e) => Err(e.into()),
        }
    }

    fn validate_name(archive_name: &String) -> Result<()> {
        if archive_name.len() > 100 {
            return Err(CompressError::ArchiveNameTooLong.into());
        }

        Ok(())
    }

    fn do_compress(
        src_dir: &Path,
        dst_file: &Path,
        options: SimpleFileOptions,
    ) -> Result<()> {
        // TODO consider moving the creation of walkdir to the compress function
        let dir_to_compress = Path::new(&src_dir);
        let walk_dir = WalkDir::new(dir_to_compress);
        let entry_iter = walk_dir.into_iter().filter_map(|entry| entry.ok());

        let zip_file = File::create(&dst_file)?;
        let mut zip_archive = ZipWriter::new(zip_file);

        let mut buffer = vec![];
        for entry in entry_iter {
            let path = entry.path();
            let name = path.strip_prefix(src_dir)?;
            let path_as_string = name
                .to_str()
                .map(str::to_owned)
                .with_context(|| format!("{name:?} Is a Non UTF-8 Path"))?;

            if path.is_file() {
                println!("adding file {path:?} as {name:?} ...");
                zip_archive.start_file(path_as_string, options)?;
                let mut file = File::open(path)?;

                // TODO consider using this to allocate the buffer vector maybe?
                // could make the reading more efficient
                //let file_len = file.metadata().unwrap().len();

                file.read_to_end(&mut buffer)?;
                zip_archive.write_all(&buffer)?;
                buffer.clear();
            } else if !name.as_os_str().is_empty() {
                // Only if not root! Avoids path spec / warning
                // and mapname conversion failed error on unzip
                println!("adding dir {path_as_string:?} as {name:?} ...");
                zip_archive.add_directory(path_as_string, options)?;
            }
        }

        zip_archive.finish()?;
        Ok(())
    }
    pub fn compress(archive_path: String, archive_name: String, src_dir: String) -> Result<()> {
        Self::validate_name(&archive_name)?;

        // validate src_dir path
        Self::validate_path(&src_dir)?;

        // validate destination path for archive
        Self::validate_path(&archive_path)?;

        let dst_archive_full_path = format!("{}{}", archive_path, archive_name);
        println!("archive_path: {dst_archive_full_path}");
        let dst_archive_full_path = Path::new(&dst_archive_full_path);

        let src_dir = Path::new(&src_dir);

        // TODO consider making this an argument
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        Self::do_compress(src_dir, dst_archive_full_path, options)?;

        Ok(())
    }
}
