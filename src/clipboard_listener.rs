use std::{
    os::windows::ffi::OsStrExt,
    error::Error,
    ffi::OsStr,
    ptr
};
use winapi::{
    shared::{
        minwindef::LPVOID,
        windef::HWND
    },
    um::winuser::{
        AddClipboardFormatListener, CreateWindowExW, 
        GetMessageW, RemoveClipboardFormatListener, 
        CW_USEDEFAULT, MSG, WM_CLIPBOARDUPDATE, 
        WS_OVERLAPPEDWINDOW, TranslateMessage,
        DispatchMessageW
    }
};
use crate::{
    helpers,
    clipboard_service::WM_APP_QUIT
};

pub struct ClipboardListener(HWND);

impl Drop for ClipboardListener {
    fn drop(&mut self) {
        unsafe  {
            RemoveClipboardFormatListener(self.0);
        }
    }
}

impl ClipboardListener {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        unsafe {
            let class_name: Vec<u16> = OsStr::new("STATIC")
                .encode_wide()
                .chain(Some(0))
                .collect();
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                ptr::null(),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut() as LPVOID
            );

            if hwnd.is_null() {
                return Err("Failed to create window".into());
            } else {
                return Ok(ClipboardListener(hwnd));
            }
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.0
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        unsafe  {
            if self.0.is_null() {
                return Err("Failed to create window".into());
            }

            if AddClipboardFormatListener(self.0) == 0 {
                return Err("Failed to add clipboard format listener".into());
            }

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                if msg.message == WM_CLIPBOARDUPDATE {
                    let _ = helpers::process_clipboard();
                } else if msg.message == WM_APP_QUIT {
                    break;
                }

                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        Ok(())
    }
}
