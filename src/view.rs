use gpui::{
    Context, CursorStyle, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
    div, hsla, uniform_list,
};
use std::ops::Range;

use crate::models::{ClipboardEntry, History};

pub struct View {
    snapshot: Vec<ClipboardEntry>,
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
                    cx.processor(|this, range: Range<usize>, _window, _cx| {
                        range
                            .map(|i| {
                                let entry = this.snapshot.get(i).unwrap();
                                let content = entry.content.clone();
                                let content = if content.len() > 25 {
                                    format!("{}...", &content[0..25])
                                } else {
                                    content
                                };
                                let timestamp =
                                    entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                                div()
                                    .py_1()
                                    .px_2()
                                    .flex_col()
                                    .w_full()
                                    .rounded_lg()
                                    .hover(|style| {
                                        style
                                            .bg(hsla(0.0, 0.0, 0.6, 0.1))
                                            .cursor(CursorStyle::PointingHand)
                                    })
                                    .child(content)
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
