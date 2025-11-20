use crate::models::{ClipboardEntry, ClipboardItem, History};
use chrono::Local;
use dispatch2::run_on_main;
use gpui::{App, AsyncApp, ImageFormat, http_client::Url};
use objc2_app_kit::{
    NSBitmapImageFileType, NSBitmapImageRep, NSPasteboard, NSPasteboardType,
    NSPasteboardTypeFileURL, NSPasteboardTypePNG, NSPasteboardTypeString, NSPasteboardTypeTIFF,
    NSPasteboardTypeURL, NSWorkspace,
};
use objc2_foundation::{NSDictionary, NSSize, NSString, NSURL};
use std::{path::PathBuf, time::Duration};

const NSPASTEBOARD_TYPE_JPEG: &str = "public.jpeg";
const NSPASTEBOARD_TYPE_GIF: &str = "com.compuserve.gif";

fn get_file_icon(path: PathBuf) -> Option<Vec<u8>> {
    run_on_main(move |_mtm| {
        let workspace = unsafe { NSWorkspace::sharedWorkspace() };
        let ns_image =
            unsafe { workspace.iconForFile(&NSString::from_str(path.to_str().unwrap())) };
        let target_size = NSSize::new(48.0, 48.0);
        unsafe { ns_image.setSize(target_size) };

        if let Some(tiff_data) = unsafe { ns_image.TIFFRepresentation() } {
            if let Some(bitmap) = unsafe { NSBitmapImageRep::imageRepWithData(&tiff_data) } {
                if let Some(png_data) = unsafe {
                    bitmap.representationUsingType_properties(
                        NSBitmapImageFileType::PNG,
                        &NSDictionary::new(),
                    )
                } {
                    Some(png_data.to_vec())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn get_pasteboard_change_count() -> isize {
    run_on_main(|_mtm| unsafe { NSPasteboard::generalPasteboard().changeCount() })
}

fn get_pasteboard_items() -> Option<Vec<ClipboardItem>> {
    run_on_main(|_mtm| {
        let items = unsafe { NSPasteboard::generalPasteboard().pasteboardItems() };

        if let Some(items) = items {
            let collected = items
                .iter()
                .flat_map(|item| {
                    unsafe { item.types() }
                        .iter()
                        .filter_map(move |t| {
                            let t: &NSPasteboardType = t.as_ref();
                            if unsafe { t == NSPasteboardTypeString } {
                                if let Some(ns_string) =
                                    unsafe { item.stringForType(NSPasteboardTypeString) }
                                {
                                    Some(ClipboardItem::Text(ns_string.to_string()))
                                } else {
                                    None
                                }
                            } else if t == unsafe { NSPasteboardTypePNG } {
                                if let Some(ns_data) =
                                    unsafe { item.dataForType(NSPasteboardTypePNG) }
                                {
                                    Some(ClipboardItem::Image {
                                        bytes: ns_data.to_vec(),
                                        format: ImageFormat::Png,
                                    })
                                } else {
                                    None
                                }
                            } else if t == unsafe { NSPasteboardTypeTIFF } {
                                if let Some(ns_data) =
                                    unsafe { item.dataForType(NSPasteboardTypeTIFF) }
                                {
                                    Some(ClipboardItem::Image {
                                        bytes: ns_data.to_vec(),
                                        format: ImageFormat::Tiff,
                                    })
                                } else {
                                    None
                                }
                            } else if t == unsafe { NSPasteboardTypeURL } {
                                if let Some(ns_string) =
                                    unsafe { item.stringForType(NSPasteboardTypeURL) }
                                {
                                    match Url::parse(&ns_string.to_string()) {
                                        Ok(url) => Some(ClipboardItem::Url(url)),
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                }
                            } else if t == unsafe { NSPasteboardTypeFileURL } {
                                if let Some(ns_string) =
                                    unsafe { item.stringForType(NSPasteboardTypeFileURL) }
                                {
                                    if let Some(url) = unsafe { NSURL::URLWithString(&ns_string) } {
                                        if let Some(path) = unsafe { url.path() } {
                                            let path_buf = PathBuf::from(path.to_string());
                                            Some(ClipboardItem::File {
                                                path: path_buf.clone(),
                                                icon_bytes: get_file_icon(path_buf),
                                            })
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
            Some(collected)
        } else {
            None
        }
    })
}

pub struct ClipboardMonitor {}

impl ClipboardMonitor {
    pub fn spawn<F>(cx: &mut App, history: History, on_change: F)
    where
        F: FnMut(&mut AsyncApp) + 'static,
    {
        cx.spawn({
            let history = history.clone();
            move |cx: &mut AsyncApp| {
                let mut cx = cx.clone();
                let history = history.clone();
                let mut on_change = on_change;
                async move {
                    let mut last_change_count = get_pasteboard_change_count();
                    loop {
                        cx.background_executor()
                            .timer(Duration::from_millis(100))
                            .await;
                        let current_change_count = get_pasteboard_change_count();
                        if current_change_count != last_change_count {
                            if let Some(items) = get_pasteboard_items() {
                                let mut history = history.lock().unwrap();
                                if let Some(i) =
                                    history.iter().position(|entry| entry.items == items)
                                {
                                    let mut old = history.remove(i);
                                    old.timestamp = Local::now();
                                    history.insert(0, old);
                                } else {
                                    history.insert(
                                        0,
                                        ClipboardEntry {
                                            items,
                                            timestamp: Local::now(),
                                        },
                                    );
                                }
                                if history.len() > 20 {
                                    history.truncate(20);
                                }
                            }
                            on_change(&mut cx);
                            last_change_count = current_change_count;
                        }
                    }
                }
            }
        })
        .detach();
    }
}
