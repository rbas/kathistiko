use std::{path::PathBuf, time::Duration};

use log::{error, info};
use thiserror::Error;
use tokio::task::JoinHandle;

use crate::{inbound::http::AppState, settings::DisplaySettings};

use super::{bitmap, snapshot, BITMAP_SIZE};

#[derive(Debug, Error)]
enum RenderError {
    #[error(transparent)]
    Snapshot(#[from] snapshot::SnapshotError),
    #[error(transparent)]
    Bitmap(#[from] bitmap::BitmapError),
    #[error("display rendering task was cancelled")]
    Cancelled,
}

pub fn spawn_display_worker(state: AppState) -> Option<JoinHandle<()>> {
    let settings = state.settings.display.clone();
    if !settings.enabled {
        info!("Display rendering is disabled");
        return None;
    }

    Some(tokio::spawn(async move {
        let refresh_interval = Duration::from_secs(settings.refresh_interval_seconds.max(1));
        let retry_interval = Duration::from_secs(30).min(refresh_interval);
        let mut next_run = Duration::ZERO;

        loop {
            tokio::time::sleep(next_run).await;

            match render_once(settings.clone()).await {
                Ok(bitmap) => {
                    info!("Published {}-byte display bitmap", BITMAP_SIZE);
                    state.display.publish(bitmap).await;
                    next_run = refresh_interval;
                }
                Err(error) => {
                    error!("Display rendering failed: {error:#}");
                    next_run = retry_interval;
                }
            }
        }
    }))
}

async fn render_once(settings: DisplaySettings) -> Result<Vec<u8>, RenderError> {
    tokio::task::spawn_blocking(move || {
        let output_directory = PathBuf::from(&settings.output_directory);
        let snapshot_path = output_directory.join("snapshot.png");
        let converted_path = output_directory.join("converted_image.png");

        snapshot::render(&settings, PathBuf::from("snapshot.png").as_path())?;
        bitmap::build_bitmap(&snapshot_path, &converted_path).map_err(RenderError::from)
    })
    .await
    .map_err(|_| RenderError::Cancelled)?
}
