use std::{
    error::Error,
    sync::mpsc,
    thread
};
use winapi::{
    shared::{
        minwindef::{UINT},
        windef::HWND
    },
    um::winuser::{
        PostMessageW,
        WM_APP
    }
};
use crate::clipboard_listener::ClipboardListener;

pub const WM_APP_QUIT: UINT = WM_APP + 1;

pub struct ClipboardService(HWND);

impl ClipboardService {
    pub fn start() -> Result<Self, Box<dyn Error>> {
        let (hwnd_tx, hwnd_rx) = mpsc::channel::<isize>();

        thread::spawn(move || {
            let mut listener = ClipboardListener::new().unwrap();

            hwnd_tx.send(listener.hwnd() as isize).unwrap();
            let _ = listener.run();
        });
        let hwnd = hwnd_rx.recv()? as HWND;

        Ok(ClipboardService(hwnd))
    }

    pub fn stop(&self) {
        unsafe {
            PostMessageW(self.0, WM_APP_QUIT, 0, 0);
        }
    }
}
