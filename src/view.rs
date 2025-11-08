use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div, hsla, px,
    uniform_list,
};
use std::ops::Range;

use crate::models::{ClipboardEntry, History};

pub struct View {
    snapshot: Vec<ClipboardEntry>,
}

impl View {
    pub const MIN_HEIGHT: f32 = 100.0;
    pub const MAX_HEIGHT: f32 = 300.0;
    pub const ROW_HEIGHT: f32 = 36.0;

    pub fn new() -> Self {
        Self {
            snapshot: Vec::new(),
        }
    }

    pub fn update_snapshot(&mut self, history: History) {
        let locked = history.lock().unwrap();
        self.snapshot = locked.clone();
    }

    pub fn panel_height_for_len(len: usize) -> f32 {
        let base = if len == 0 {
            Self::ROW_HEIGHT
        } else {
            (len as f32) * Self::ROW_HEIGHT
        };

        base.clamp(Self::MIN_HEIGHT, Self::MAX_HEIGHT)
    }
}

impl Render for View {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let len = self.snapshot.len();
        let panel_height = Self::panel_height_for_len(len);

        div()
            .flex_col()
            .gap_2()
            .h(px(panel_height))
            .p_4()
            .text_color(hsla(0.0, 0.0, 0.95, 1.0))
            .bg(hsla(0.0, 0.0, 0.08, 0.5))
            .text_xs()
            .w_full()
            .child(
                uniform_list(
                    "history",
                    self.snapshot.len(),
                    cx.processor(|this, range: Range<usize>, _window, _cx| {
                        range
                            .map(|i| {
                                let entry = this.snapshot.get(i).unwrap();
                                let content = entry.content.clone();
                                let content = if content.len() > 15 {
                                    format!("{}...", &content[0..15])
                                } else {
                                    content
                                };
                                div()
                                    .h(px(Self::ROW_HEIGHT))
                                    .rounded_sm()
                                    .hover(|style| style.bg(hsla(0.0, 0.0, 0.15, 0.5)))
                                    .flex()
                                    .items_center()
                                    .child(format!("{content}"))
                            })
                            .collect()
                    }),
                )
                .flex_grow(),
            )
    }
}
