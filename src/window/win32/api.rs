use std::{
    ffi::OsString,
    os::{raw::c_void, windows::ffi::OsStrExt},
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
        CW_USEDEFAULT, CreateWindowExW, GWLP_USERDATA, GetWindowLongPtrW, IDC_ARROW, LoadCursorW,
        RegisterClassW, SetWindowLongPtrW, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    },
};

pub unsafe fn set_user_data<T>(handle: HWND, data: *mut T) {
    unsafe {
        SetWindowLongPtrW(handle, GWLP_USERDATA, data as isize);
    }
}

pub unsafe fn get_user_data(handle: HWND) -> isize {
    unsafe { GetWindowLongPtrW(handle, GWLP_USERDATA) }
}

pub fn wide_string(str: &str) -> Vec<u16> {
    OsString::from(str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[inline]
fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

type WndProc = unsafe extern "system" fn(HWND, u32, usize, isize) -> isize;

pub unsafe fn register_class(name: &str, proc: WndProc) {
    let name = wide_string(name);

    let wc = WNDCLASSW {
        lpszClassName: name.as_ptr(),
        hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) },
        lpfnWndProc: Some(proc),
        hInstance: unsafe { GetModuleHandleW(ptr::null_mut()) },
        hbrBackground: unsafe { CreateSolidBrush(rgb(20, 20, 20)) },
        ..Default::default()
    };

    unsafe {
        RegisterClassW(&wc);
    }
}

pub unsafe fn create_window<T>(user_data: *mut T) {
    let instance = unsafe { GetModuleHandleW(ptr::null_mut()) };
    let class_name = wide_string("fever-class");
    let name = wide_string("");

    unsafe {
        CreateWindowExW(
            0,                        // estilos estendidos
            class_name.as_ptr(),      // nome da classe
            name.as_ptr(),            // nome da janela
            WS_OVERLAPPEDWINDOW,      // estilo
            CW_USEDEFAULT,            // x
            CW_USEDEFAULT,            // y
            CW_USEDEFAULT,            // altura
            CW_USEDEFAULT,            // largura
            ptr::null_mut(),          // pai
            ptr::null_mut(),          // menu
            instance,                 // instância
            user_data as *mut c_void, // userdata
        );
    }
}

pub fn is_dark_mode() -> bool {
    let mut value: u32 = 1;
    let mut size = std::mem::size_of::<u32>() as u32;

    let path = wide_string("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let key = wide_string("AppsUseLightTheme");

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
