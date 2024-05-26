use std::fs::OpenOptions;
use std::io::{BufWriter, Seek, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::IrisError;

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum FileType {
    Directory,
    File,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileMetadata {
    dest_filename: PathBuf,
    file_type: FileType,
    size: u64,
}

impl FileMetadata {
    pub fn new(filename: PathBuf, file_type: FileType, file_size: u64) -> Self {
        Self {
            dest_filename: filename,
            file_type,
            size: file_size,
        }
    }

    pub fn get_filename(&self) -> &PathBuf {
        &self.dest_filename
    }

    pub fn get_file_type(&self) -> FileType {
        self.file_type
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }
}

/// A thin wrapper around std::fs::File.
pub struct File {
    file: std::fs::File,
    writer: BufWriter<std::fs::File>,
    path: PathBuf,
}

impl File {
    pub fn open_new_in_append(path: PathBuf) -> Result<Self, IrisError> {
        let file = open_file(&path, true)?;
        let file_handle_copy = file
            .try_clone()
            .map_err(|_| IrisError::PermissionsUserIOError(path.display().to_string()))?;

        let writer = BufWriter::new(file_handle_copy);
        tracing::debug!("opened {path:?}");

        Ok(Self { file, writer, path })
    }

    pub fn open_in_append(path: PathBuf) -> Result<Self, IrisError> {
        let file = open_file(&path, false)?;
        let file_handle_copy = file
            .try_clone()
            .map_err(|_| IrisError::PermissionsUserIOError(path.display().to_string()))?;

        let writer = BufWriter::new(file_handle_copy);
        tracing::debug!("opened {path:?}");

        Ok(Self { file, writer, path })
    }

    pub fn open_in_overwrite(path: PathBuf) -> Result<Self, IrisError> {
        let mut file = open_file(&path, false)?;
        file.set_len(0).map_err(|_| {
            IrisError::PermissionsUserIOError(format!("unable to write to '{}'", path.display()))
        })?;
        file.rewind().map_err(|_| {
            IrisError::PermissionsUserIOError(format!("unable to write to '{}'", path.display()))
        })?;
        let file_handle_copy = file
            .try_clone()
            .map_err(|_| IrisError::PermissionsUserIOError(path.display().to_string()))?;

        let writer = BufWriter::new(file_handle_copy);
        tracing::debug!("opened {path:?}");

        Ok(Self { file, writer, path })
    }

    pub fn get_size(&self) -> Result<u64, IrisError> {
        Ok(self
            .file
            .metadata()
            .map_err(|_| IrisError::PermissionsUserIOError(self.path.display().to_string()))?
            .len())
    }

    pub fn write_chunk(&mut self, plaintext: &[u8]) -> Result<(), IrisError> {
        self.writer
            .write_all(plaintext)
            .map_err(|_| IrisError::PermissionsUserIOError(self.path.display().to_string()))
    }
}

fn open_file(path: &Path, create_new: bool) -> Result<std::fs::File, IrisError> {
    if create_new {
        OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(path)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::AlreadyExists => {
                    IrisError::AlreadyExistsUserIOError(path.display().to_string())
                }
                _ => IrisError::PermissionsUserIOError(path.display().to_string()),
            })
    } else {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| IrisError::PermissionsUserIOError(path.display().to_string()))
    }
}
