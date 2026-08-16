mod bitmap;
mod snapshot;
mod worker;

use std::sync::Arc;

use tokio::sync::RwLock;

pub use worker::spawn_display_worker;

pub const DISPLAY_WIDTH: u32 = 800;
pub const DISPLAY_HEIGHT: u32 = 480;
pub const BITMAP_SIZE: usize = (DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize) / 8;

#[derive(Debug, Clone, Default)]
pub struct DisplayStore {
    bitmap: Arc<RwLock<Option<Vec<u8>>>>,
}

impl DisplayStore {
    pub async fn latest(&self) -> Option<Vec<u8>> {
        self.bitmap.read().await.clone()
    }

    async fn publish(&self, bitmap: Vec<u8>) {
        *self.bitmap.write().await = Some(bitmap);
    }
}
