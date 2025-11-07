use gpui::{
    App, Context, IntoElement, ParentElement, Render, Styled, Window, div, px, uniform_list,
};
use std::ops::Range;

use crate::models::{ClipboardEntry, History};

pub struct View {
    snapshot: Vec<ClipboardEntry>,
}

impl View {
    pub fn new() -> Self {
        let snapshot: Vec<ClipboardEntry> = Vec::new();
        Self { snapshot }
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
            .rounded_3xl()
            .min_h(px(100.0))
            .w(px(200.0))
            .child(uniform_list(
                "history",
                self.snapshot.len(),
                cx.processor(|this, range: Range<usize>, _window, _cx| {
                    range
                        .map(|i| {
                            let entry = this.snapshot.get(i).unwrap();
                            let content = entry.content.clone();
                            div().child(format!("{content}"))
                        })
                        .collect()
                }),
            ))
    }
}
