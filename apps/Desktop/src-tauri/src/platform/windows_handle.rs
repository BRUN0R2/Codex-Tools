use windows::Win32::Foundation::{CloseHandle, HANDLE};

pub struct OwnedWindowsHandle {
    handle: HANDLE,
}

impl OwnedWindowsHandle {
    pub fn new(handle: HANDLE) -> Option<Self> {
        if handle.is_invalid() {
            return None;
        }

        Some(Self { handle })
    }

    pub fn raw(&self) -> HANDLE {
        self.handle
    }
}

impl Drop for OwnedWindowsHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
