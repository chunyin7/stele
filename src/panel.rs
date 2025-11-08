use gpui::{
    App, AppContext, Bounds, WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind,
    WindowOptions, px, size,
};

use crate::{models::History, view::View};

pub struct Panel {
    window: WindowHandle<View>,
    history: History,
}

impl Panel {
    const WIDTH: f32 = 220.0;
    const HEIGHT: f32 = 250.0;

    pub fn new(cx: &mut App, history: History) -> Self {
        let window = Self::open_window(cx, history.clone());
        Self { window, history }
    }

    fn open_window(cx: &mut App, history: History) -> WindowHandle<View> {
        let bounds = Bounds::centered(None, size(px(Self::WIDTH), px(Self::HEIGHT)), cx);
        cx.open_window(
            WindowOptions {
                titlebar: None,
                is_movable: false,
                kind: WindowKind::PopUp,
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            },
            move |_window, cx| {
                let history = history.clone();
                cx.new(|_cx| {
                    let mut view = View::new();
                    view.update_snapshot(history.clone());
                    view
                })
            },
        )
        .unwrap()
    }

    pub fn hide(&mut self, cx: &mut App) {
        let _ = self.window.update(cx, |_view, window, _cx| window.remove_window());
    }

    pub fn show(&mut self, cx: &mut App) {
        *self = Self::new(cx, self.history.clone());
    }

    pub fn toggle(&mut self, cx: &mut App) {
        if let Some(_) = self.window.is_active(cx) {
            self.hide(cx);
        } else {
            self.show(cx);
        }
    }

    pub fn sync_history(&mut self, cx: &mut App) {
        let history = self.history.clone();
        let _ = self.window.update(cx, |view, window, _cx| {
            view.update_snapshot(history);
            window.refresh();
        });
    }
}
