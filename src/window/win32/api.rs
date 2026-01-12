use std::{
    ffi::{OsStr, c_void},
    os::windows::ffi::OsStrExt,
    ptr,
};

use windows_sys::Win32::{
    Foundation::HWND,
    Graphics::{Dwm::DwmSetWindowAttribute, Gdi::CreateSolidBrush},
    System::{
        LibraryLoader::GetModuleHandleW,
        Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW},
    },
    UI::WindowsAndMessaging::{
        CREATESTRUCTW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW,
        GWL_USERDATA, GWLP_USERDATA, GetWindowLongPtrW, IDC_ARROW, LoadCursorW, MSG, PM_REMOVE,
        PeekMessageW, RegisterClassW, SW_SHOW, SetWindowLongPtrW, ShowWindow, TranslateMessage,
        WM_CREATE, WM_NCCREATE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    },
};

#[derive(Clone, Copy)]
pub struct SyncHandle(pub HWND);

unsafe impl Send for SyncHandle {}

unsafe impl Sync for SyncHandle {}

pub struct WString {
    data: Vec<u16>,
}

pub struct EventData {
    wp: usize,
    lp: isize,
}

type WndProc = unsafe extern "system" fn(HWND, u32, usize, isize) -> isize;

impl WString {
    pub fn new<'a>(str: &'a str) -> Self {
        Self {
            data: OsStr::new(str)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect(),
        }
    }

    pub fn as_ptr(&self) -> *const u16 {
        return self.data.as_ptr();
    }
}

impl<'a> Into<WString> for &'a str {
    fn into(self) -> WString {
        return WString::new(self);
    }
}

impl Into<WString> for String {
    fn into(self) -> WString {
        return WString::new(self.as_str());
    }
}

pub struct ClassAttributes<'a> {
    pub name: &'a WString,
    pub proc: unsafe extern "system" fn(*mut c_void, u32, usize, isize) -> isize,
}

#[inline]
fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

pub unsafe fn use_window_procedure(proc: WndProc) {
    let class_name = WString::new("fever_class");
    let instance = unsafe { GetModuleHandleW(ptr::null_mut()) };

    let mut wc = WNDCLASSW {
        hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) },
        lpfnWndProc: Some(proc),
        lpszClassName: class_name.as_ptr(),
        hInstance: instance,
        hbrBackground: unsafe { CreateSolidBrush(rgb(20, 20, 20)) },
        ..Default::default()
    };

    unsafe {
        RegisterClassW(&wc);
    }
}

pub unsafe fn create_window(user_data: *mut c_void) {
    let window_title = WString::new("Iniciando...");
    let class_name = WString::new("fever_class");
    let instance = unsafe { GetModuleHandleW(ptr::null_mut()) };

    unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_title.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            ptr::null_mut(),
            ptr::null_mut(),
            instance,
            user_data,
        );
    }
}

pub unsafe fn def_wind_proc(handle: HWND, msg: u32, wp: usize, lp: isize) -> isize {
    return unsafe { DefWindowProcW(handle, msg, wp, lp) };
}

pub unsafe fn set_user_data<T>(handle: HWND, data: *mut T) {
    unsafe {
        SetWindowLongPtrW(handle, GWLP_USERDATA, data as isize);
    }
}

pub unsafe fn get_user_data(handle: HWND) -> isize {
    unsafe { GetWindowLongPtrW(handle, GWL_USERDATA) }
}

pub unsafe fn get_creation_struct(lp: isize) -> *mut CREATESTRUCTW {
    return lp as *mut CREATESTRUCTW;
}

pub struct Message {
    inner: MSG,
}

pub unsafe fn peek_message() -> bool {
    unsafe {
        let mut msg = MSG::default();
        let has_message = PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0;
        if (has_message) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        return has_message;
    }
}

pub fn is_dark_mode() -> bool {
    let mut value: u32 = 1;
    let mut size = std::mem::size_of::<u32>() as u32;

    let path = WString::new("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let key = WString::new("AppsUseLightTheme");

    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            key.as_ptr(),
            RRF_RT_REG_DWORD,
            ptr::null_mut(),
            &mut value as *mut _ as *mut _,
            &mut size,
        )
    };

    return status == 0 && value == 0;
}

pub unsafe fn enable_dark_titlebar(handle: HWND) {
    let enable: i32 = 1;

    unsafe {
        DwmSetWindowAttribute(
            handle,
            20,
            &enable as *const _ as *const _,
            std::mem::size_of::<i32>() as u32,
        );

        DwmSetWindowAttribute(
            handle,
            19,
            &enable as *const _ as *const _,
            std::mem::size_of::<i32>() as u32,
        );
    }
}