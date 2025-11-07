use gpui::{App, WindowHandle};

use crate::view::View;

pub struct Panel {
    pub window: WindowHandle<View>,
}

impl Panel {
    pub fn new(cx: &mut App) -> Self {
        Self {}
    }
}
