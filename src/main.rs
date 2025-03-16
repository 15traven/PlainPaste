// #![windows_subsystem = "windows"]

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
mod autolaunch;

use clipboard_listener::ClipboardListener;

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
    let autolaunch_item = CheckMenuItem::new("Run at startup", true, true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let _ = tray_menu.append_items(&[
        &autolaunch_item,
        &PredefinedMenuItem::separator(),
        &quit_item
    ]);

    let mut _window: Option<Window> = None;
    let mut tray_icon: Option<TrayIcon> = None;

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
                    autolaunch_item.set_checked(autolaunch::is_enabled().unwrap());
                }
                
                std::thread::spawn(move || {
                    let mut clipboard_listener = ClipboardListener::new().unwrap();
                    let _ = clipboard_listener.run();
                });
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
