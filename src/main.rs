#![windows_subsystem = "windows"]

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Theme, Window, WindowBuilder}
};
use tray_icon::{
    menu::{
        Menu, MenuEvent, MenuItem, 
        CheckMenuItem, PredefinedMenuItem
    },
    TrayIcon, TrayIconBuilder
};

mod helpers;
mod clipboard_listener;
mod clipboard_service;
mod autolaunch;

use clipboard_service::ClipboardService;

enum UserEvent {
    MenuEvent(MenuEvent)
}

fn main() {
    let light_icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/light_icon.png");
    let light_icon = helpers::load_icon(std::path::Path::new(light_icon_path));

    let dark_icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/dark_icon.png");
    let dark_icon = helpers::load_icon(std::path::Path::new(dark_icon_path));

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let tray_menu = Menu::new();
    let autoformat_item = CheckMenuItem::new("Automatically clear formating", true, true, None);
    let autolaunch_item = CheckMenuItem::new("Run at startup", true, true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let _ = tray_menu.append_items(&[
        &autoformat_item,
        &PredefinedMenuItem::separator(),
        &autolaunch_item,
        &quit_item
    ]);

    let mut _window: Option<Window> = None;
    let mut tray_icon: Option<TrayIcon> = None;

    let mut clipboard_service: Option<ClipboardService> = None;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(tao::event::StartCause::Init) => {
                _window = Some(
                    WindowBuilder::new()
                        .with_visible(false)
                        .build(&event_loop)
                        .unwrap()
                );

                tray_icon = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(tray_menu.clone()))
                        .build()
                        .unwrap()
                );
                let _ = match _window.as_ref().unwrap().theme() {
                    Theme::Dark => tray_icon.as_ref().unwrap().set_icon(Some(light_icon.clone())),
                    Theme::Light => tray_icon.as_ref().unwrap().set_icon(Some(dark_icon.clone())),
                    _ => Ok(())
                };

                if autolaunch::register().is_ok() {
                    let is_enabled = autolaunch::is_enabled();
                    if is_enabled.is_err() {
                        let _ = autolaunch::enable();
                        autolaunch_item.set_checked(true);
                    } else {
                        autolaunch_item.set_checked(is_enabled.unwrap());
                    }
                }
                
                clipboard_service = Some(ClipboardService::start().unwrap());
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::ThemeChanged(theme) => {
                    let _ = match theme {
                        Theme::Dark => tray_icon.as_ref().unwrap().set_icon(Some(light_icon.clone())),
                        Theme::Light => tray_icon.as_ref().unwrap().set_icon(Some(dark_icon.clone())),
                        _ => Ok(())
                    };
                }
                _ => {}
            }

            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                if event.id == autoformat_item.id() {
                    if autoformat_item.is_checked() {
                        clipboard_service = Some(ClipboardService::start().unwrap());
                    } else {
                        clipboard_service.as_ref().unwrap().stop();
                    }
                }

                if event.id == autolaunch_item.id() {
                    let _ = match autolaunch::is_enabled().unwrap() {
                        true => autolaunch::disable(),
                        false => autolaunch::enable()
                    };
                }

                if event.id == quit_item.id() {
                    tray_icon.take();
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
