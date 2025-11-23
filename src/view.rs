use dispatch2::run_on_main;
use gpui::{
    App, Context, CursorStyle, FocusHandle, Image, ImageFormat, ImageSource, InteractiveElement,
    IntoElement, KeyDownEvent, ObjectFit, Overflow, ParentElement, Render, ScrollHandle,
    StatefulInteractiveElement, Styled, StyledImage, Window, div, hsla, img,
    prelude::FluentBuilder, px, svg, uniform_list,
};
use objc2_app_kit::{
    NSPasteboard, NSPasteboardTypeFileURL, NSPasteboardTypePNG, NSPasteboardTypeString,
    NSPasteboardTypeTIFF, NSPasteboardTypeURL, NSWorkspace,
};
use objc2_foundation::{NSData, NSString};
use std::{
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::models::{ClipboardEntry, ClipboardItem, History};

pub struct View {
    snapshot: Vec<ClipboardEntry>,
    cur_idx: usize,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
}

fn render_item(item: ClipboardItem) -> impl IntoElement {
    match item {
        ClipboardItem::Text(text) => {
            let text = if text.len() > 25 {
                format!("{}...", &text[0..25])
            } else {
                text.clone()
            };

            div().child(text)
        }
        ClipboardItem::Url(url) => {
            let url_string = url.to_string();
            let url_string = if url_string.len() > 25 {
                format!("{}...", &url_string[0..25])
            } else {
                url_string
            };

            div()
                .underline()
                .text_color(hsla(240.0, 0.93, 0.83, 1.0))
                .text_decoration_color(hsla(240.0, 0.93, 0.83, 1.0))
                .child(url_string)
        }
        ClipboardItem::File { path, icon_bytes } => {
            if let Some(icon_bytes) = icon_bytes {
                let image = Arc::new(Image::from_bytes(ImageFormat::Png, icon_bytes));
                div().child(img(image).size_12())
            } else {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                div().child(filename)
            }
        }
        ClipboardItem::Image { bytes, format } => {
            let image = Arc::new(Image::from_bytes(format, bytes));
            div().child(
                img(image)
                    .max_h_32()
                    .max_w(px(180.0))
                    .object_fit(ObjectFit::Contain),
            )
        }
    }
}

fn copy_entry_to_clipboard(entry: ClipboardEntry) {
    let items = entry.items.clone();

    run_on_main(move |_mtm| {
        let pasteboard = unsafe { NSPasteboard::generalPasteboard() };
        unsafe { pasteboard.clearContents() };
        items.iter().for_each(|item| match item {
            ClipboardItem::Text(text) => {
                let nsstring = NSString::from_str(text);
                unsafe { pasteboard.setString_forType(&nsstring, NSPasteboardTypeString) };
            }
            ClipboardItem::Url(url) => {
                let nsstring = NSString::from_str(url.as_str());
                unsafe { pasteboard.setString_forType(&nsstring, NSPasteboardTypeURL) };
            }
            ClipboardItem::Image { bytes, format } => {
                let nsdata = NSData::from_vec(bytes.clone());
                let nsimagetype = match format {
                    ImageFormat::Png => Some(unsafe { NSPasteboardTypePNG }),
                    ImageFormat::Tiff => Some(unsafe { NSPasteboardTypeTIFF }),
                    _ => None,
                };
                if let Some(nsimagetype) = nsimagetype {
                    unsafe { pasteboard.setData_forType(Some(&nsdata), nsimagetype) };
                }
            }
            ClipboardItem::File { path, icon_bytes } => {
                let url = match url::Url::from_file_path(path) {
                    Ok(url) => url,
                    Err(_) => return,
                };
                let nsstring = NSString::from_str(url.as_str());
                unsafe { pasteboard.setString_forType(&nsstring, NSPasteboardTypeFileURL) };
            }
        });
    })
}

impl View {
    pub fn new(cx: &mut App) -> Self {
        Self {
            snapshot: Vec::new(),
            cur_idx: 0,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
        }
    }

    fn move_down(&mut self) {
        self.cur_idx = (self.cur_idx + 1) % self.snapshot.len();
    }

    fn move_up(&mut self) {
        self.cur_idx = (self.cur_idx + self.snapshot.len() - 1) % self.snapshot.len();
    }

    pub fn update_snapshot(&mut self, history: History) {
        let locked = history.lock().unwrap();
        self.snapshot = locked.clone();
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for View {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_col()
            .track_focus(&self.focus_handle())
            .gap_2()
            .h_full()
            .w_full()
            .text_color(hsla(0.0, 0.0, 0.9, 1.0))
            .bg(hsla(0.0, 0.0, 0.08, 0.5))
            .text_xs()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "j" => {
                        this.move_down();
                        this.scroll_handle.scroll_to_item(this.cur_idx);
                        cx.notify();
                    }
                    "k" => {
                        this.move_up();
                        this.scroll_handle.scroll_to_item(this.cur_idx);
                        cx.notify();
                    }
                    "enter" => {
                        let entry = this.snapshot.get(this.cur_idx).unwrap();
                        copy_entry_to_clipboard(entry.clone());
                        window.remove_window();
                    }
                    "escape" => {
                        cx.hide();
                        window.remove_window();
                    }
                    _ => {}
                }
            }))
            .p_2()
            .id("history")
            .overflow_y_scroll()
            .track_scroll(&self.scroll_handle.clone())
            .children(self.snapshot.iter().enumerate().map(|(i, entry)| {
                let items = entry.items.clone();
                let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                div()
                    .py_1()
                    .px_2()
                    .flex_col()
                    .w_full()
                    .when(self.cur_idx == i, |style| {
                        style.bg(hsla(0.0, 0.0, 0.6, 0.1))
                    })
                    .id(("outer", i))
                    .on_click(cx.listener(move |this, _event, window, _cx| {
                        let entry = this.snapshot.get(i).unwrap();
                        copy_entry_to_clipboard(entry.clone());
                        window.remove_window();
                    }))
                    .rounded_lg()
                    .hover(|style| {
                        style
                            .bg(hsla(0.0, 0.0, 0.6, 0.1))
                            .cursor(CursorStyle::PointingHand)
                    })
                    .child(
                        div()
                            .flex_col()
                            .children(items.iter().map(|item| render_item(item.clone())))
                            .child(div().text_color(hsla(0.0, 0.0, 0.9, 0.8)).child(timestamp)),
                    )
            }))
    }
}
