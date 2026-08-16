mod bitmap;
mod snapshot;
mod worker;

use std::sync::Arc;

use sha2::{Digest, Sha256};
use tokio::sync::RwLock;

pub use worker::spawn_display_worker;

pub const DISPLAY_WIDTH: u32 = 800;
pub const DISPLAY_HEIGHT: u32 = 480;
pub const BITMAP_SIZE: usize = (DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize) / 8;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DisplayImage {
    bitmap: Vec<u8>,
    etag: String,
}

impl DisplayImage {
    fn new(bitmap: Vec<u8>) -> Self {
        let digest = Sha256::digest(&bitmap);
        Self {
            bitmap,
            etag: format!("\"{digest:x}\""),
        }
    }

    pub fn bitmap(&self) -> &[u8] {
        &self.bitmap
    }

    pub fn etag(&self) -> &str {
        &self.etag
    }
}

#[derive(Debug, Clone, Default)]
pub struct DisplayStore {
    image: Arc<RwLock<Option<Arc<DisplayImage>>>>,
}

impl DisplayStore {
    pub(crate) async fn latest(&self) -> Option<Arc<DisplayImage>> {
        self.image.read().await.clone()
    }

    async fn publish(&self, bitmap: Vec<u8>) {
        *self.image.write().await = Some(Arc::new(DisplayImage::new(bitmap)));
    }
}

#[cfg(test)]
mod tests {
    use super::{DisplayImage, DisplayStore};

    #[test]
    fn etag_is_stable_for_identical_bitmap_bytes() {
        let first = DisplayImage::new(vec![0xAA, 0x55]);
        let second = DisplayImage::new(vec![0xAA, 0x55]);

        assert_eq!(first.etag(), second.etag());
        assert!(first.etag().starts_with('"'));
        assert!(first.etag().ends_with('"'));
    }

    #[test]
    fn etag_changes_with_bitmap_bytes() {
        let first = DisplayImage::new(vec![0xAA, 0x55]);
        let second = DisplayImage::new(vec![0xAA, 0x54]);

        assert_ne!(first.etag(), second.etag());
    }

    #[tokio::test]
    async fn store_publishes_bitmap_and_etag_together() {
        let store = DisplayStore::default();
        store.publish(vec![1, 2, 3]).await;

        let image = store.latest().await.unwrap();
        assert_eq!(image.bitmap(), [1, 2, 3]);
        assert!(!image.etag().is_empty());
    }
}
