use crate::modules::models::{
    RunLocalScriptInput, ScriptEntryDetail, ScriptEntrySummary, ScriptExecutionResult,
};
use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
use tokio::process::Command;

#[derive(Clone)]
pub struct ScriptWorkspace {
    root_dir: PathBuf,
}

impl ScriptWorkspace {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn list_scripts(&self) -> Result<Vec<ScriptEntrySummary>> {
        let mut entries = Vec::new();

        if !self.root_dir.exists() {
            fs::create_dir_all(&self.root_dir).with_context(|| {
                format!(
                    "failed to create script workspace root: {}",
                    self.root_dir.display()
                )
            })?;
        }

        self.walk_dir(&self.root_dir, &mut entries)?;
        entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        Ok(entries)
    }

    pub fn read_script(&self, path: &str) -> Result<ScriptEntryDetail> {
        let target = self.resolve_script_path(path)?;
        let summary = self.build_summary(&target)?;
        let content = fs::read_to_string(&target)
            .with_context(|| format!("failed to read script content: {}", target.display()))?;
        let kind = script_kind(&target);
        Ok(ScriptEntryDetail {
            suggested_remote_command: build_remote_command(kind, &content),
            local_runner: build_local_runner(&target),
            summary,
            content,
        })
    }

    pub async fn run_local_script(
        &self,
        input: RunLocalScriptInput,
    ) -> Result<ScriptExecutionResult> {
        let target = self.resolve_script_path(&input.path)?;
        let args = input.args.unwrap_or_default();
        let (program, base_args) = command_for_script(&target)?;

        let mut command = Command::new(program);
        command.args(base_args.iter().map(String::as_str));
        command.arg(target.as_os_str());
        command.args(args.iter().map(String::as_str));
        command.current_dir(&self.root_dir);

        let output = command.output().await.with_context(|| {
            format!("failed to run local script: {}", target.display())
        })?;

        Ok(ScriptExecutionResult {
            command: render_command(program, &base_args, &target, &args),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    fn walk_dir(&self, dir: &Path, entries: &mut Vec<ScriptEntrySummary>) -> Result<()> {
        for entry in fs::read_dir(dir)
            .with_context(|| format!("failed to read script directory: {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.walk_dir(&path, entries)?;
                continue;
            }
            if !is_supported_script(&path) {
                continue;
            }
            entries.push(self.build_summary(&path)?);
        }

        Ok(())
    }

    fn build_summary(&self, path: &Path) -> Result<ScriptEntrySummary> {
        let metadata = fs::metadata(path)
            .with_context(|| format!("failed to stat script file: {}", path.display()))?;
        let relative_path = path
            .strip_prefix(&self.root_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();
        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or_default();
        Ok(ScriptEntrySummary {
            id: relative_path.clone(),
            name: path
                .file_name()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_else(|| relative_path.clone()),
            path: path.to_string_lossy().into_owned(),
            kind: script_kind(path).into(),
            relative_path,
            size_bytes: metadata.len(),
            modified_at,
        })
    }

    fn resolve_script_path(&self, path: &str) -> Result<PathBuf> {
        let candidate = PathBuf::from(path);
        let target = if candidate.is_absolute() {
            candidate
        } else {
            self.root_dir.join(candidate)
        };
        let normalized = target
            .canonicalize()
            .with_context(|| format!("script path not found: {}", target.display()))?;
        let canonical_root = self.root_dir.canonicalize().with_context(|| {
            format!(
                "failed to canonicalize script workspace root: {}",
                self.root_dir.display()
            )
        })?;
        if !normalized.starts_with(&canonical_root) {
            bail!("script path escapes workspace root");
        }
        if !is_supported_script(&normalized) {
            bail!("unsupported script type");
        }
        Ok(normalized)
    }
}

fn is_supported_script(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("py" | "sh"))
}

fn script_kind(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("py") => "python",
        Some("sh") => "shell",
        _ => "unknown",
    }
}

fn command_for_script(path: &Path) -> Result<(&'static str, Vec<String>)> {
    match script_kind(path) {
        "python" => Ok(("python3", Vec::new())),
        "shell" => Ok(("bash", Vec::new())),
        _ => bail!("unsupported script type"),
    }
}

fn build_remote_command(kind: &str, content: &str) -> String {
    match kind {
        "python" => format!(
            "python3 - <<'PY'\n{}\nPY",
            content.trim_end_matches('\n')
        ),
        "shell" => format!(
            "bash -s <<'SH'\n{}\nSH",
            content.trim_end_matches('\n')
        ),
        _ => content.to_owned(),
    }
}

fn build_local_runner(path: &Path) -> String {
    let quoted = shell_quote(path.to_string_lossy().as_ref());
    match script_kind(path) {
        "python" => format!("python3 {quoted}"),
        "shell" => format!("bash {quoted}"),
        _ => quoted,
    }
}

fn render_command(program: &str, base_args: &[String], target: &Path, args: &[String]) -> String {
    let mut parts = vec![program.to_owned()];
    parts.extend(base_args.iter().cloned());
    parts.push(shell_quote(target.to_string_lossy().as_ref()));
    parts.extend(args.iter().map(|arg| shell_quote(arg)));
    parts.join(" ")
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".into();
    }
    if value
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || matches!(char, '/' | '.' | '_' | '-' | ':'))
    {
        return value.into();
    }
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
