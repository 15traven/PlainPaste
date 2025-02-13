// #![windows_subsystem = "windows"]

use clipboard_listener::ClipboardListener;
use tao::{
    event::Event,
    event_loop::{
        ControlFlow,
        EventLoopBuilder
    }
};
use tray_icon::{
    menu::{
        Menu, MenuEvent, MenuItem
    },
    TrayIcon, TrayIconBuilder
};

mod helpers;
mod clipboard_listener;

enum UserEvent {
    MenuEvent(MenuEvent)
}

fn main() {
    let light_icon_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/light_icon.png");
    let light_icon = helpers::load_icon(std::path::Path::new(light_icon_path));

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    let _ = tray_menu.append_items(&[
        &quit_item
    ]);

    let mut tray_icon: Option<TrayIcon> = None;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(tao::event::StartCause::Init) => {
                tray_icon = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(tray_menu.clone()))
                        .with_icon(light_icon.clone())
                        .build()
                        .unwrap()
                );
                
                std::thread::spawn(move || {
                    let mut clipboard_listener = ClipboardListener::new().unwrap();
                    let _ = clipboard_listener.run();
                });
            }
            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                if event.id == quit_item.id() {
                    tray_icon.take();
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
