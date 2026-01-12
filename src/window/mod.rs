#![allow(unused)]

#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "windows")]
use win32 as platform;

pub use platform::Window;
pub use platform::WindowServer;

#[derive(Debug, PartialEq)]
pub enum MouseState {
    Down,
    Up,
}

#[derive(Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Debug)]
pub enum WindowEvent {
    Click {
        x: i32,
        y: i32,
        state: MouseState,
        button: MouseButton,
    },

    MouseMove {
        x: i32,
        y: i32,
    },

    Resize {
        width: u32,
        height: u32,
    },

    Close,
}

pub struct WindowMessage {
    pub window_id: u32,
    pub event: WindowEvent,
}

pub struct WindowOptions {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "Fever".into(),
            width: 1200,
            height: 675,
        }
    }
}
