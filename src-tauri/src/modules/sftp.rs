use crate::modules::models::{
    CreateSftpDirectoryInput, DeleteSftpEntryInput, DownloadSftpFileInput,
    ListSftpDirectoryInput, SftpDirectorySnapshot, SftpEntryNode, SftpOperationResult,
    UploadSftpFileInput,
};
use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

#[derive(Clone)]
pub struct SftpWorkspace {
    root_dir: PathBuf,
}

impl SftpWorkspace {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn list_directory(
        &self,
        input: ListSftpDirectoryInput,
    ) -> Result<SftpDirectorySnapshot> {
        let target = self.resolve_directory(input.path.as_deref())?;
        let mut entries = Vec::new();

        for entry in fs::read_dir(&target)
            .with_context(|| format!("failed to read directory: {}", target.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry
                .metadata()
                .with_context(|| format!("failed to read metadata: {}", path.display()))?;
            entries.push(SftpEntryNode {
                name: entry.file_name().to_string_lossy().into_owned(),
                path: path.to_string_lossy().into_owned(),
                kind: if metadata.is_dir() {
                    "directory".into()
                } else {
                    "file".into()
                },
                size_bytes: if metadata.is_file() { metadata.len() } else { 0 },
                permissions: permissions_string(&metadata),
                modified_at: metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs() as i64)
                    .unwrap_or_default(),
            });
        }

        entries.sort_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .reverse()
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });

        Ok(SftpDirectorySnapshot {
            root_path: self.root_dir.to_string_lossy().into_owned(),
            current_path: target.to_string_lossy().into_owned(),
            total_entries: entries.len(),
            entries,
        })
    }

    pub fn create_directory(
        &self,
        input: CreateSftpDirectoryInput,
    ) -> Result<SftpOperationResult> {
        let parent = self.resolve_directory(Some(input.parent_path.as_str()))?;
        let name = sanitize_entry_name(&input.name)?;
        let target = parent.join(&name);
        fs::create_dir_all(&target)
            .with_context(|| format!("failed to create directory: {}", target.display()))?;
        Ok(SftpOperationResult {
            action: "mkdir".into(),
            source_path: None,
            target_path: target.to_string_lossy().into_owned(),
            bytes_transferred: 0,
        })
    }

    pub fn delete_entry(
        &self,
        input: DeleteSftpEntryInput,
    ) -> Result<SftpOperationResult> {
        let target = self.resolve_entry(input.path.as_str())?;
        if target.is_dir() {
            fs::remove_dir_all(&target)
                .with_context(|| format!("failed to delete directory: {}", target.display()))?;
        } else {
            fs::remove_file(&target)
                .with_context(|| format!("failed to delete file: {}", target.display()))?;
        }
        Ok(SftpOperationResult {
            action: "delete".into(),
            source_path: Some(target.to_string_lossy().into_owned()),
            target_path: target.to_string_lossy().into_owned(),
            bytes_transferred: 0,
        })
    }

    pub fn upload_file(
        &self,
        input: UploadSftpFileInput,
    ) -> Result<SftpOperationResult> {
        let source = PathBuf::from(&input.source_path)
            .canonicalize()
            .with_context(|| format!("upload source not found: {}", input.source_path))?;
        if !source.is_file() {
            bail!("upload source is not a file");
        }

        let target_directory =
            self.resolve_directory(Some(input.target_directory.as_str()))?;
        let target_name = input
            .target_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(sanitize_entry_name)
            .transpose()?
            .unwrap_or_else(|| {
                source
                    .file_name()
                    .map(|value| value.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "uploaded-file".into())
            });
        let target = target_directory.join(target_name);
        let bytes = fs::copy(&source, &target).with_context(|| {
            format!(
                "failed to copy upload source {} -> {}",
                source.display(),
                target.display()
            )
        })?;
        Ok(SftpOperationResult {
            action: "upload".into(),
            source_path: Some(source.to_string_lossy().into_owned()),
            target_path: target.to_string_lossy().into_owned(),
            bytes_transferred: bytes,
        })
    }

    pub fn download_file(
        &self,
        input: DownloadSftpFileInput,
    ) -> Result<SftpOperationResult> {
        let source = self.resolve_entry(input.source_path.as_str())?;
        if !source.is_file() {
            bail!("download source is not a file");
        }

        let destination = PathBuf::from(&input.destination_path);
        let target = if destination.is_dir()
            || input.destination_path.ends_with(std::path::MAIN_SEPARATOR)
        {
            destination.join(
                source
                    .file_name()
                    .map(|value| value.to_os_string())
                    .unwrap_or_default(),
            )
        } else {
            destination
        };
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create download destination: {}", parent.display())
            })?;
        }

        let bytes = fs::copy(&source, &target).with_context(|| {
            format!(
                "failed to copy download source {} -> {}",
                source.display(),
                target.display()
            )
        })?;
        Ok(SftpOperationResult {
            action: "download".into(),
            source_path: Some(source.to_string_lossy().into_owned()),
            target_path: target.to_string_lossy().into_owned(),
            bytes_transferred: bytes,
        })
    }

    fn resolve_directory(&self, requested: Option<&str>) -> Result<PathBuf> {
        let candidate = requested
            .map(PathBuf::from)
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| self.root_dir.clone());

        let resolved = if candidate.is_absolute() {
            candidate
        } else {
            self.root_dir.join(candidate)
        };

        let normalized = resolved
            .canonicalize()
            .with_context(|| format!("directory not found: {}", resolved.display()))?;
        let canonical_root = self.root_dir.canonicalize().with_context(|| {
            format!(
                "failed to canonicalize sftp workspace root: {}",
                self.root_dir.display()
            )
        })?;

        if !normalized.starts_with(&canonical_root) {
            bail!("directory escapes sftp workspace root");
        }
        if !normalized.is_dir() {
            bail!("target path is not a directory");
        }

        Ok(normalized)
    }

    fn resolve_entry(&self, requested: &str) -> Result<PathBuf> {
        let candidate = PathBuf::from(requested);
        let resolved = if candidate.is_absolute() {
            candidate
        } else {
            self.root_dir.join(candidate)
        };

        let normalized = resolved
            .canonicalize()
            .with_context(|| format!("entry not found: {}", resolved.display()))?;
        let canonical_root = self.root_dir.canonicalize().with_context(|| {
            format!(
                "failed to canonicalize sftp workspace root: {}",
                self.root_dir.display()
            )
        })?;
        if !normalized.starts_with(&canonical_root) {
            bail!("entry escapes sftp workspace root");
        }
        Ok(normalized)
    }
}

fn permissions_string(metadata: &fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        format!("{:o}", metadata.permissions().mode() & 0o777)
    }

    #[cfg(not(unix))]
    {
        if metadata.permissions().readonly() {
            "readonly".into()
        } else {
            "rw".into()
        }
    }
}

fn sanitize_entry_name(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("entry name cannot be empty");
    }
    if trimmed == "." || trimmed == ".." {
        bail!("entry name cannot be dot path");
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        bail!("entry name cannot contain path separators");
    }
    Ok(trimmed.to_owned())
}
