use std::{path::Path, process::Command};

use thiserror::Error;

use crate::settings::DisplaySettings;

#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("failed to start snapshot container")]
    Start(#[source] std::io::Error),
    #[error("snapshot container exited with status {0}")]
    Failed(std::process::ExitStatus),
}

pub fn render(settings: &DisplaySettings, filename: &Path) -> Result<(), SnapshotError> {
    let status = Command::new(&settings.container_binary)
        .current_dir(&settings.working_directory)
        .args(["compose", "run", "-e"])
        .arg(format!("URL={}", settings.source_url))
        .args(["-e"])
        .arg(format!("OUTPUT_FILENAME={}", filename.display()))
        .args(["--rm", "snapshot"])
        .status()
        .map_err(SnapshotError::Start)?;

    if status.success() {
        Ok(())
    } else {
        Err(SnapshotError::Failed(status))
    }
}
