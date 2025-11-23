use dispatch2::run_on_main;
use gpui::{
    App, AppContext, Bounds, WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowKind,
    WindowOptions, point, px, size,
};
use objc2_app_kit::{NSEvent, NSScreen};
use objc2_foundation::{NSArray, NSString};

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
        let mouse_pos = run_on_main(|_mtm| unsafe { NSEvent::mouseLocation() });

        let displays = cx.displays();
        let active = displays.iter().find(move |display| {
            let bounds = display.bounds();
            mouse_pos.x >= bounds.origin.x.to_f64()
                && mouse_pos.x <= (bounds.origin.x + bounds.size.width).to_f64()
                && mouse_pos.y >= bounds.origin.y.to_f64()
                && mouse_pos.y <= (bounds.origin.y + bounds.size.height).to_f64()
        });

        let bounds = if let Some(display) = active {
            // appkit gives relative to bottom of screen, gpui expects relative to top of screen
            let bounds = display.bounds();
            let flipped_y =
                2.0 * bounds.origin.y.to_f64() + bounds.size.height.to_f64() - mouse_pos.y;
            Some((mouse_pos.x, flipped_y));
            Bounds::new(
                point(px(mouse_pos.x as f32), px(flipped_y as f32)),
                size(px(Self::WIDTH), px(Self::HEIGHT)),
            )
        } else {
            Bounds::centered(None, size(px(Self::WIDTH), px(Self::HEIGHT)), cx)
        };

        let window = cx
            .open_window(
                WindowOptions {
                    titlebar: None,
                    is_movable: false,
                    kind: WindowKind::PopUp,
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: WindowBackgroundAppearance::Blurred,
                    display_id: if let Some(display) = active {
                        Some(display.id())
                    } else {
                        None
                    },
                    ..Default::default()
                },
                move |_window, cx| {
                    let history = history.clone();
                    cx.new(|cx| {
                        let mut view = View::new(cx);
                        view.update_snapshot(history.clone());
                        view
                    })
                },
            )
            .unwrap();

        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle());
            })
            .unwrap();

        window
    }

    pub fn hide(&mut self, cx: &mut App) {
        let _ = self.window.update(cx, |_view, window, cx| {
            cx.hide();
            window.remove_window();
        });
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
