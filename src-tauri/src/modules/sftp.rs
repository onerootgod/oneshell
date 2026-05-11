use crate::modules::models::{ListSftpDirectoryInput, SftpDirectorySnapshot, SftpEntryNode};
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
