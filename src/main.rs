use std::sync::{Arc, Mutex};

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use gpui::{App, Application};

use crate::{models::History, monitor::ClipboardMonitor};

mod models;
mod monitor;
mod panel;
mod view;

fn main() {
    Application::new().run(|cx: &mut App| {
        let history: History = Arc::new(Mutex::new(Vec::new()));
        ClipboardMonitor::spawn(cx, history.clone(), || {});

        let manager = Box::leak(Box::new(
            GlobalHotKeyManager::new().expect("Failed to create global hotkey manager"),
        ));
        let hotkey = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyV);
        manager.register(hotkey).unwrap();

        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.state == HotKeyState::Pressed {
                todo!("show panel");
            }
        }));
    });
}
