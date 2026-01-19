#![allow(unused)]

use raw_window_handle as rwh;

#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "windows")]
use win32 as platform;

pub struct Window {
    inner: platform::Window,
}

impl Window {
    pub fn handle(&self) -> NativeHandle {
        NativeHandle {
            inner: self.inner.get_native(),
        }
    }
}

pub struct WindowServer {
    inner: platform::WindowServer,
}

impl WindowServer {
    pub fn connect() -> (Self, WindowClient) {
        let (server, client) = platform::WindowServer::connect();

        return (
            WindowServer { inner: server },
            WindowClient { inner: client },
        );
    }

    pub fn run(mut self) {
        self.inner.run();
    }
}

pub struct WindowClient {
    inner: platform::WindowClient,
}

impl WindowClient {
    pub fn new_window(&self, options: WindowOptions) -> Window {
        Window {
            inner: self.inner.new_window(options),
        }
    }

	pub fn poll_events<H: FnMut(Input)>(&self, handler: H) {
		self.inner.poll_events(handler);
	}

    pub fn terminate(&mut self) {
        return self.inner.terminate();
    }
}

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

pub enum Input {
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
    pub event: Input,
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

pub struct NativeHandle {
    inner: platform::NativeHandle,
}

impl rwh::HasWindowHandle for NativeHandle {
    fn window_handle(&self) -> Result<rwh::WindowHandle<'_>, rwh::HandleError> {
        self.inner.window_handle()
    }
}

impl rwh::HasDisplayHandle for NativeHandle {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        self.inner.display_handle()
    }
}