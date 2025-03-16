use std::ptr;
use winapi::{
    shared::minwindef::FALSE,
    um::{
        winbase::{
            GlobalFree, GlobalLock,
            GlobalUnlock, GlobalAlloc
        },
        winuser::{
            OpenClipboard, CloseClipboard,
            GetClipboardData, SetClipboardData,
            EmptyClipboard, CF_UNICODETEXT
        },
        winnt::HANDLE
    }
};

pub fn process_clipboard() -> Result<(), String> {
    unsafe {
        if OpenClipboard(ptr::null_mut()) == FALSE {
            return Err("Failed to open clipboard".into());
        }

        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle.is_null() {
            CloseClipboard();
            return Err("Failed to get clipboard data".into());
        }

        let ptr = GlobalLock(handle) as *const u16;
        if ptr.is_null() {
            CloseClipboard();
            return Err("Failed to lock global memory".into());
        }

        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let clipboard_text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));

        GlobalUnlock(handle);
        if EmptyClipboard() == FALSE {
            CloseClipboard();
            return Err("Failed to empty clipboard".into());
        }

        let encoded_text: Vec<u16> = clipboard_text.encode_utf16().chain(std::iter::once(0)).collect();
        let size = encoded_text.len() * std::mem::size_of::<u16>();

        let h_mem: HANDLE = GlobalAlloc(0x0002, size);
        if h_mem.is_null() {
            CloseClipboard();
            return Err("Failed to allocate global memory".into());
        }

        let ptr = GlobalLock(h_mem) as *mut u16;
        if ptr.is_null() {
            GlobalFree(h_mem);
            CloseClipboard();
            return Err("Failed to lock global memory".into());
        }

        ptr::copy_nonoverlapping(encoded_text.as_ptr(), ptr, encoded_text.len());
        GlobalUnlock(h_mem);

        if SetClipboardData(CF_UNICODETEXT, h_mem).is_null() {
            GlobalFree(h_mem);
            CloseClipboard();
            return Err("Failed to set clipboard data".into());
        }

        CloseClipboard();
        Ok(())
    }
}

pub fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    tray_icon::Icon::from_rgba(
        icon_rgba, 
        icon_width, 
        icon_height
    ).expect("Failed to open icon")
}