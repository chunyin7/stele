use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use gpui::{App, AppContext, Application, AsyncApp};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;
use tray_icon::{
    Icon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

use crate::{models::History, monitor::ClipboardMonitor, panel::Panel};

mod models;
mod monitor;
mod panel;
mod view;

fn main() {
    Application::new().run(|cx: &mut App| {
        // Set as accessory app (no dock icon) after gpui initializes
        if let Some(mtm) = MainThreadMarker::new() {
            let app = NSApplication::sharedApplication(mtm);
            app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        }

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

        let icon_bytes = include_bytes!("../assets/stele.png");
        let image = image::load_from_memory(icon_bytes).unwrap().to_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        let tray_menu = Menu::new();
        tray_menu
            .append_items(&[
                &MenuItem::with_id("name", "stele v0.1.2", false, None),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id("quit", "Quit", true, None),
            ])
            .unwrap();
        let _tray = Box::leak(Box::new(
            TrayIconBuilder::new()
                .with_icon(icon)
                .with_icon_as_template(true)
                .with_menu(Box::new(tray_menu))
                .build()
                .unwrap(),
        ));
        let tray_receiver = MenuEvent::receiver();

        cx.spawn({
            let tray_receiver = tray_receiver;
            move |cx: &mut AsyncApp| {
                let cx = cx.clone();
                async move {
                    loop {
                        while let Ok(event) = tray_receiver.try_recv() {
                            if event.id().0.as_str() == "quit" {
                                cx.update(|cx| cx.quit()).unwrap();
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
