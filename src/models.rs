use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Local};
use gpui::{ImageFormat, http_client::Url};

#[derive(Clone)]
pub enum ClipboardItem {
    Text(String),
    Url(Url),
    File(PathBuf),
    Image { bytes: Vec<u8>, format: ImageFormat },
}

#[derive(Clone)]
pub struct ClipboardEntry {
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub items: Vec<ClipboardItem>,
}

pub type History = Arc<Mutex<Vec<ClipboardEntry>>>;
