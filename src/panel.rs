use gpui::{
    App, AppContext, Bounds, WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind,
    WindowOptions, px, size,
};

use crate::{models::History, view::View};

pub struct Panel {
    window: Option<WindowHandle<View>>,
    history: History,
}

impl Panel {
    const WIDTH: f32 = 220.0;

    pub fn new(cx: &mut App, history: History) -> Self {
        let mut panel = Self {
            window: None,
            history,
        };
        panel.show(cx);
        panel
    }

    fn desired_height(&self) -> f32 {
        let history = self.history.lock().unwrap();
        View::panel_height_for_len(history.len())
    }

    fn open_window(&self, cx: &mut App) -> WindowHandle<View> {
        let height = self.desired_height();
        let bounds = Bounds::centered(None, size(px(Self::WIDTH), px(height)), cx);
        let history = self.history.clone();
        let window = cx
            .open_window(
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
            .unwrap();

        let target_size = size(px(Self::WIDTH), px(height));
        let _ = window.update(cx, |_view, window, _| {
            window.resize(target_size);
        });

        window
    }

    fn refresh_view(&self, cx: &mut App) {
        if let Some(window) = &self.window {
            let target_height = self.desired_height();
            let target_size = size(px(Self::WIDTH), px(target_height));
            let history = self.history.clone();
            let _ = window.update(cx, |view, window, ctx| {
                view.update_snapshot(history.clone());
                ctx.notify();
                window.resize(target_size);
                window.refresh();
            });
        }
    }

    pub fn hide(&mut self, cx: &mut App) {
        if let Some(window) = self.window.take() {
            let _ = window.update(cx, |_view, window, _cx| {
                window.remove_window();
            });
        }
    }

    pub fn show(&mut self, cx: &mut App) {
        if self.window.is_none() {
            self.window = Some(self.open_window(cx));
        } else {
            self.refresh_view(cx);
        }
    }

    pub fn toggle(&mut self, cx: &mut App) {
        if self.window.is_some() {
            self.hide(cx);
        } else {
            self.show(cx);
        }
    }

    pub fn sync_history(&mut self, cx: &mut App) {
        self.refresh_view(cx);
    }
}
