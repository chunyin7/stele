use crate::models::{ClipboardEntry, History};
use chrono::Local;
use dispatch2::run_on_main;
use gpui::{App, Context};
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use std::{thread, time::Duration};

fn get_pasteboard_change_count() -> isize {
    run_on_main(|_mtm| unsafe { NSPasteboard::generalPasteboard().changeCount() })
}

fn get_pasteboard_content() -> Option<String> {
    run_on_main(|_mtm| unsafe {
        NSPasteboard::generalPasteboard()
            .stringForType(NSPasteboardTypeString)
            .map(|ns_string| ns_string.to_string())
    })
}

pub struct ClipboardMonitor {}

impl ClipboardMonitor {
    pub fn spawn<F>(cx: &mut App, history: History, mut on_change: F)
    where
        F: FnMut() + Send + 'static,
    {
        cx.spawn(async move |cx| {
            let mut last_change_count = get_pasteboard_change_count();
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                let current_change_count = get_pasteboard_change_count();
                if current_change_count != last_change_count {
                    if let Some(content) = get_pasteboard_content() {
                        let mut history = history.lock().unwrap();
                        history.insert(
                            0,
                            ClipboardEntry {
                                content,
                                timestamp: Local::now(),
                            },
                        );
                        if history.len() > 20 {
                            history.truncate(20);
                        }
                    }
                    on_change();
                    last_change_count = current_change_count;
                }
            }
        })
        .detach();
    }
}
