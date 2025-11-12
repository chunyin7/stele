use dispatch2::run_on_main;
use gpui::{
    Context, CursorStyle, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div, hsla, uniform_list,
};
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::NSString;
use std::ops::Range;

use crate::models::{ClipboardEntry, ClipboardItem, History};

pub struct View {
    snapshot: Vec<ClipboardEntry>,
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
        ClipboardItem::File(path) => {
            // todo
            div()
        }
        ClipboardItem::Image { bytes, format } => {
            // todo
            div()
        }
    }
}

impl View {
    pub fn new() -> Self {
        Self {
            snapshot: Vec::new(),
        }
    }

    pub fn update_snapshot(&mut self, history: History) {
        let locked = history.lock().unwrap();
        self.snapshot = locked.clone();
    }

    fn copy_entry_to_clipboard(&self, entry: ClipboardEntry) {
        let content = entry.content.clone();

        run_on_main(move |_mtm| unsafe {
            let pasteboard = NSPasteboard::generalPasteboard();
            pasteboard.clearContents();
            let nsstring = NSString::from_str(&content);
            pasteboard.setString_forType(&nsstring, NSPasteboardTypeString);
        })
    }
}

impl Render for View {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_col()
            .gap_2()
            .h_full()
            .w_full()
            .text_color(hsla(0.0, 0.0, 0.9, 1.0))
            .bg(hsla(0.0, 0.0, 0.08, 0.5))
            .text_xs()
            .p_2()
            .child(
                uniform_list(
                    "history",
                    self.snapshot.len(),
                    cx.processor(|this, range: Range<usize>, _window, cx| {
                        range
                            .map(|i| {
                                let entry = this.snapshot.get(i).unwrap();
                                let items = entry.items.clone();
                                let timestamp =
                                    entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                                div()
                                    .py_1()
                                    .px_2()
                                    .flex_col()
                                    .w_full()
                                    .id(i)
                                    .on_click(cx.listener(move |this, _event, window, _cx| {
                                        let entry = this.snapshot.get(i).unwrap();
                                        this.copy_entry_to_clipboard(entry.clone());
                                        window.remove_window();
                                    }))
                                    .rounded_lg()
                                    .hover(|style| {
                                        style
                                            .bg(hsla(0.0, 0.0, 0.6, 0.1))
                                            .cursor(CursorStyle::PointingHand)
                                    })
                                    .child(uniform_list(
                                        i,
                                        items.len(),
                                        cx.processor(
                                            move |_this, range: Range<usize>, _window, _cx| {
                                                range
                                                    .map(|j| render_item(items[j].clone()))
                                                    .collect()
                                            },
                                        ),
                                    ))
                                    .child(
                                        div().text_color(hsla(0.0, 0.0, 0.9, 0.8)).child(timestamp),
                                    )
                            })
                            .collect()
                    }),
                )
                .h_full(),
            )
    }
}
