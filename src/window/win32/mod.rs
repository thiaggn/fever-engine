mod api;

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    ffi::c_void,
    ptr,
    rc::Rc,
    sync::{Arc, Mutex, OnceLock},
};

use windows_sys::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{
        AdjustWindowRectEx, SW_SHOW, SWP_NOMOVE, SWP_NOZORDER, SetWindowPos, SetWindowTextW, ShowWindow, WM_CLOSE, WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_NCCREATE, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SIZE, WS_OVERLAPPEDWINDOW
    },
};

use crate::window::{
    MouseButton, MouseState, WindowEvent, WindowMessage, WindowOptions,
    win32::api::{SyncHandle, WString},
};

struct WindowState {
    running: bool,
    width: u32,
    height: u32,
    visible: bool,
}

pub struct Window {
    id: u32,
    handle: api::SyncHandle,
    state: Arc<Mutex<WindowState>>,
}

impl Window {
    pub fn id(&self) -> u32 {
        return self.id;
    }

    pub fn set_size(&self, width: u32, height: u32) {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };

        unsafe {
            AdjustWindowRectEx(&mut rect, WS_OVERLAPPEDWINDOW, 0, 0);
        }

        let w_width = rect.right - rect.left;
        let w_height = rect.bottom - rect.top;

        unsafe {
            SetWindowPos(
                self.handle.0,
                std::ptr::null_mut(),
                0,
                0,
                w_width,
                w_height,
                SWP_NOMOVE | SWP_NOZORDER,
            );
        }
    }

    pub fn set_title(&self, title: &str) {
        let title: WString = title.into();

        unsafe {
            SetWindowTextW(self.handle.0, title.as_ptr());
        }
    }

    pub fn show(&self) {
        let mut state = self.state.lock().unwrap();
        state.visible = true;

        unsafe {
            ShowWindow(self.handle.0, SW_SHOW);
        }
    }
}

pub struct WindowServer {
    inner: Rc<ServerInner>,
}

impl WindowServer {
    pub fn start() -> Self {
        unsafe {
            api::use_window_procedure(wndproc);
        }
        return Self {
            inner: Rc::new(ServerInner::new()),
        };
    }

    pub fn new_window(&self, options: WindowOptions) -> Window {
        let mut ctx = CreateCtx {
            window: None,
            options: options,
            server: self.inner.clone(),
        };
        unsafe {
            api::create_window(&mut ctx as *mut _ as *mut _);
        }
        return ctx.window.unwrap();
    }

    pub fn poll(&self) -> Option<WindowMessage> {
        unsafe {
            api::peek_message();
        }
        return self.inner.consume_next_event();
    }
}

struct ServerInner {
    next_id: Cell<u32>,
    queue: RefCell<VecDeque<WindowMessage>>,
}

impl ServerInner {
    fn new() -> Self {
        return Self {
            next_id: Cell::new(1),
            queue: RefCell::new(VecDeque::with_capacity(64)),
        };
    }

    fn enqueue_event(&self, event: WindowMessage) {
        self.queue.borrow_mut().push_back(event);
    }

    fn consume_next_event(&self) -> Option<WindowMessage> {
        return self.queue.borrow_mut().pop_front();
    }

    fn produce_id(&self) -> u32 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        return id;
    }
}

unsafe extern "system" fn window_event(event: NativeEvent, ctx: &RunCtx) -> isize {
    match event.id {
        WM_SIZE => event.unhandled(),

        WM_LBUTTONDOWN => {
            ctx.send_event(WindowEvent::Click {
                x: event.x_param() as i32,
                y: event.y_param() as i32,
                state: MouseState::Down,
                button: MouseButton::Left,
            });
            return 0;
        }

        WM_MBUTTONDOWN => {
            ctx.send_event(WindowEvent::Click {
                x: event.x_param() as i32,
                y: event.y_param() as i32,
                state: MouseState::Down,
                button: MouseButton::Middle,
            });
            return 0;
        }

        WM_RBUTTONDOWN => {
            ctx.send_event(WindowEvent::Click {
                x: event.x_param() as i32,
                y: event.y_param() as i32,
                state: MouseState::Down,
                button: MouseButton::Right,
            });
            return 0;
        }

        WM_LBUTTONUP => {
            ctx.send_event(WindowEvent::Click {
                x: event.x_param() as i32,
                y: event.y_param() as i32,
                state: MouseState::Up,
                button: MouseButton::Left,
            });
            return 0;
        }

        WM_MBUTTONUP => {
            ctx.send_event(WindowEvent::Click {
                x: event.x_param() as i32,
                y: event.y_param() as i32,
                state: MouseState::Up,
                button: MouseButton::Middle,
            });
            return 0;
        }

        WM_RBUTTONUP => {
            ctx.send_event(WindowEvent::Click {
                x: event.x_param() as i32,
                y: event.y_param() as i32,
                state: MouseState::Up,
                button: MouseButton::Right,
            });
            return 0;
        }

		WM_CLOSE => {
			ctx.send_event(WindowEvent::Close);
			return 0;
		}

        _ => event.unhandled(),
    }
}

unsafe extern "system" fn wndproc(handle: HWND, msg: u32, wp: usize, lp: isize) -> isize {
    match msg {
        WM_NCCREATE => unsafe {
            let Some(ctx) = get_create_context(lp) else {
                return 0;
            };

            let window_id = ctx.server.produce_id();

            let state = Arc::new(Mutex::new(WindowState {
                width: ctx.options.width,
                height: ctx.options.height,
                running: true,
                visible: true,
            }));

            let run_ctx = Box::into_raw(Box::new(RunCtx {
                state: state.clone(),
                server: ctx.server.clone(),
                window_id: window_id,
            }));

            let window = Window {
				id: window_id,
                handle: api::SyncHandle(handle),
                state: state.clone(),
            };

            ctx.window = Some(window);
            api::set_user_data(handle, run_ctx);
            return 1;
        },

        WM_CREATE => unsafe {
            let Some(ctx) = get_create_context(lp) else {
                return -1;
            };

            let Some(window) = &mut ctx.window else {
                return -1;
            };

            window.set_size(ctx.options.width, ctx.options.height);
            window.set_title(ctx.options.title.as_str());

            if api::is_dark_mode() {
                api::enable_dark_titlebar(handle);
            }

            window.show();
            return 0;
        },

        WM_DESTROY => unsafe {
            let Some(ctx) = get_run_context(handle) else {
                return api::def_wind_proc(handle, msg, wp, lp);
            };

            // Garante que o run context seja dropado
            Box::from_raw(ctx);
        },
        _ => {}
    }

    unsafe {
        let Some(ctx) = get_run_context(handle) else {
            return api::def_wind_proc(handle, msg, wp, lp);
        };

        return window_event(
            NativeEvent {
                handle,
                id: msg,
                wp,
                lp,
            },
            ctx,
        );
    }
}

struct CreateCtx {
    window: Option<Window>,
    options: WindowOptions,
    server: Rc<ServerInner>,
}

unsafe fn get_create_context<'a>(lp: isize) -> Option<&'a mut CreateCtx> {
    unsafe {
        api::get_creation_struct(lp).as_mut().and_then(|ptr| {
            let p2 = ptr.lpCreateParams as *mut CreateCtx;
            p2.as_mut()
        })
    }
}

struct RunCtx {
    state: Arc<Mutex<WindowState>>,
    server: Rc<ServerInner>,
    window_id: u32,
}

impl RunCtx {
    pub fn send_event(&self, event: WindowEvent) {
        self.server.enqueue_event(WindowMessage {
            window_id: self.window_id,
            event,
        });
    }
}

unsafe fn get_run_context<'a>(handle: HWND) -> Option<&'a mut RunCtx> {
    unsafe {
        let data = api::get_user_data(handle) as *mut RunCtx;
        return data.as_mut();
    }
}

#[repr(C)]
struct NativeEvent {
    handle: HWND,
    id: u32,
    wp: usize,
    lp: isize,
}

impl NativeEvent {
    pub fn unhandled(self) -> isize {
        return unsafe { api::def_wind_proc(self.handle, self.id, self.wp, self.lp) };
    }

    pub fn x_param(&self) -> i16 {
        return (self.lp & 0xFFFF) as i16;
    }

    pub fn y_param(&self) -> i16 {
        return ((self.lp >> 16) & 0xFFFF) as i16;
    }
}
