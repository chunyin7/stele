use gpui::{
    App, AppContext, Bounds, WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind,
    WindowOptions, px, size,
};

use crate::view::View;

pub struct Panel {
    pub window: WindowHandle<View>,
}

impl Panel {
    pub fn new(cx: &mut App) -> Self {
        let bounds = Bounds::centered(None, size(px(200.0), px(100.0)), cx);
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
                |_window, cx| cx.new(|_cx| View::new()),
            )
            .unwrap();
        Self { window }
    }

    pub fn hide(&mut self, cx: &mut App) {
        let _ = self.window.update(cx, |_view, _window, cx| {
            cx.hide();
        });
    }

    pub fn show(&mut self, cx: &mut App) {
        let _ = self.window.update(cx, |_view, window, _cx| {
            window.remove_window();
        });
        *self = Self::new(cx);
    }

    pub fn toggle(&mut self, cx: &mut App) {
        if let Some(_) = self.window.is_active(cx) {
            self.hide(cx);
        } else {
            self.show(cx);
        }
    }
}
