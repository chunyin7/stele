use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use gpui::{App, AppContext, Application, AsyncApp};

use crate::{models::History, monitor::ClipboardMonitor, panel::Panel};

mod models;
mod monitor;
mod panel;
mod view;

fn main() {
    Application::new().run(|cx: &mut App| {
        let history: History = Arc::new(Mutex::new(Vec::new()));
        let panel = cx.new(|cx| Panel::new(cx, history.clone()));

        let panel_for_monitor = panel.clone();
        ClipboardMonitor::spawn(cx, history.clone(), move |cx: &mut AsyncApp| {
            let _ = panel_for_monitor.update(cx, |panel, cx| panel.sync_history(cx));
        });

        let manager = Box::leak(Box::new(
            GlobalHotKeyManager::new().expect("Failed to create global hotkey manager"),
        ));
        let hotkey = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyV);
        manager.register(hotkey).unwrap();

        let receiver = GlobalHotKeyEvent::receiver().clone();
        let panel_for_hotkey = panel.clone();

        cx.spawn({
            let panel = panel_for_hotkey;
            let receiver = receiver;
            move |cx: &mut AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        while let Ok(event) = receiver.try_recv() {
                            if event.state == HotKeyState::Pressed {
                                let _ = panel.update(&mut cx, |panel, cx| panel.toggle(cx));
                            }
                        }

                        cx.background_executor()
                            .timer(Duration::from_millis(16))
                            .await;
                    }
                }
            }
        })
        .detach();
    });
}
