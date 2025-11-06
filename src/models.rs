use std::sync::{Arc, Mutex};

use chrono::{DateTime, Local};

#[derive(Clone)]
pub struct ClipboardEntry {
    pub content: String,
    pub timestamp: DateTime<Local>,
}

pub type History = Arc<Mutex<Vec<ClipboardEntry>>>;
