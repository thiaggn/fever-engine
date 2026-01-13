#![allow(unused)]

mod api;

use std::{
    collections::VecDeque,
    num::NonZeroIsize,
    ptr,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
};

use raw_window_handle::{self as rwh, RawWindowHandle, Win32WindowHandle};

use windows_sys::Win32::{
    Foundation::{HWND, RECT, TRUE},
    UI::WindowsAndMessaging::{
        AdjustWindowRectEx, CREATESTRUCTW, DefWindowProcW, DestroyWindow, DispatchMessageW, MSG,
        PM_REMOVE, PeekMessageW, SW_SHOW, SWP_NOMOVE, SWP_NOZORDER, SetWindowPos, SetWindowTextW,
        ShowWindow, TranslateMessage, WM_CLOSE, WM_CREATE, WM_LBUTTONDOWN, WM_LBUTTONUP,
        WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_NCCREATE, WM_RBUTTONDOWN, WM_RBUTTONUP,
        WM_SIZE, WNDCLASSEXW, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    },
};

use crate::window::{MouseButton, MouseState, WindowEvent, WindowOptions};

enum ClientRequest {
    CreateWindow { options: WindowOptions },
    Terminate,
}

enum ServerResponse {
    WindowCreated { window: Window },
    None,
}

struct WindowState {
    width: u32,
    height: u32,
}

pub struct NativeHandle {
    handle: SyncHandle,
}

impl rwh::HasDisplayHandle for NativeHandle {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        Ok(rwh::DisplayHandle::windows())
    }
}

impl rwh::HasWindowHandle for NativeHandle {
    fn window_handle(&self) -> Result<rwh::WindowHandle<'_>, rwh::HandleError> {
        let handle = NonZeroIsize::new(self.handle.0 as isize)
            .expect("o handle da janela não deve ser nulo");
        let raw = RawWindowHandle::Win32(Win32WindowHandle::new(handle));
        unsafe { Ok(rwh::WindowHandle::borrow_raw(raw)) }
    }
}

#[derive(Clone, Copy)]
struct SyncHandle(HWND);

unsafe impl Sync for SyncHandle {}
unsafe impl Send for SyncHandle {}

pub struct Window {
    handle: SyncHandle,
    state: Arc<Mutex<WindowState>>,
}

impl Window {
    pub fn get_native(&self) -> NativeHandle {
        return NativeHandle {
            handle: self.handle,
        };
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
        let title = api::wide_string(title);

        unsafe {
            SetWindowTextW(self.handle.0, title.as_ptr());
        }
    }

    pub fn show(&self) {
        unsafe {
            ShowWindow(self.handle.0, SW_SHOW);
        }
    }
}

/// Estrutura que armazena eventos pendentes da janela. `WindowServer` e `WindowClient`
/// compartilham uma referência para a mesma instância de `Events`
struct Events {
    queue: VecDeque<WindowEvent>,
    resize: Option<WindowEvent>,
}

pub struct WindowServer {
    events: Arc<Mutex<Events>>,
    req: Receiver<ClientRequest>,
    res: Sender<ServerResponse>,
    client: Option<WindowClient>,
    active: bool,
}

impl WindowServer {
    pub fn new() -> Self {
        let class_name = api::wide_string("fever_window");
        unsafe {
            api::register_class("fever-class", Self::wnd_proc);
        }

        let res = mpsc::channel::<ServerResponse>();
        let req = mpsc::channel::<ClientRequest>();

        let events = Arc::new(Mutex::new(Events {
            queue: VecDeque::new(),
            resize: None,
        }));

        let client = WindowClient {
            events: events.clone(),
            req: req.0,
            res: res.1,
        };

        return Self {
            events: events.clone(),
            req: req.1,
            res: res.0,
            client: Some(client),
            active: true,
        };
    }

    pub fn run(&mut self) {
        unsafe {
            api::create_window::<()>(ptr::null_mut());
        }

        loop {
            if !self.active {
                break;
            }
            self.process_client_requests();
            unsafe {
                let mut msg = MSG::default();
                PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE);
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    pub fn client(&mut self) -> WindowClient {
        if let Some(client) = self.client.take() {
            return client;
        } else {
            panic!("apenas uma sessão pode ser criada por vez.")
        }
    }

    fn process_client_requests(&mut self) {
        if let Ok(req) = self.req.try_recv() {
            match req {
                ClientRequest::CreateWindow { options } => {
                    let window = self.create_window(options);
                    self.res
                        .send(ServerResponse::WindowCreated { window })
                        .unwrap();
                }

                ClientRequest::Terminate => {
                    self.active = false;
                }
            }
        }
    }

    fn create_window(&mut self, options: WindowOptions) -> Window {
        let mut ctx = CreateCtx {
            opt: options,
            events: self.events.clone(),
            window: None,
        };

        unsafe {
            api::create_window(&mut ctx);
        }

        return ctx.window.unwrap();
    }

    unsafe extern "system" fn wnd_proc(h: HWND, msg: u32, wp: usize, lp: isize) -> isize {
        match msg {
            WM_NCCREATE => unsafe {
                let Some(ctx) = CreateCtx::from_lparam(lp) else {
                    return -1;
                };

                let state = Arc::new(Mutex::new(WindowState {
                    width: ctx.opt.width,
                    height: ctx.opt.height,
                }));

                let run_ctx = Box::into_raw(Box::new(RunCtx {
                    state: state.clone(),
                    events: ctx.events.clone(),
                }));

                let window = Window {
                    handle: SyncHandle(h),
                    state: state.clone(),
                };

                ctx.window = Some(window);
                api::set_user_data(h, run_ctx);
                return 1;
            },

            WM_CREATE => unsafe {
                let Some(ctx) = CreateCtx::from_lparam(lp) else {
                    return -1;
                };

                let Some(window) = &mut ctx.window else {
                    return -1;
                };

                window.set_size(ctx.opt.width, ctx.opt.height);
                window.set_title(ctx.opt.title.as_str());
                if api::is_dark_mode() {
                    api::enable_dark_titlebar(h);
                }
                window.show();
                return 1;
            },

            _ => {}
        }

        unsafe {
            let Some(ctx) = RunCtx::from_hwnd(h) else {
                return DefWindowProcW(h, msg, wp, lp);
            };
            return Self::on_event(ctx, h, msg, wp, lp);
        }
    }

    unsafe extern "system" fn on_event(
        ctx: &mut RunCtx,
        h: HWND,
        msg: u32,
        wp: usize,
        lp: isize,
    ) -> isize {
        match msg {
            WM_SIZE => {
                ctx.send_event(WindowEvent::Resize {
                    width: (lp & 0xFFFF) as u32,
                    height: ((lp >> 16) & 0xFFFF) as u32,
                });
                return 0;
            }

            WM_CLOSE => {
                ctx.send_event(WindowEvent::Close);
                unsafe {
                    DestroyWindow(h);
                    Box::from_raw(ctx);
                }
                return 0;
            }

            WM_LBUTTONDOWN => {
                ctx.send_event(WindowEvent::Click {
                    x: (lp & 0xFFFF) as i32,
                    y: ((lp >> 16) & 0xFFFF) as i32,
                    state: MouseState::Down,
                    button: MouseButton::Left,
                });
                return 0;
            }

            WM_MBUTTONDOWN => {
                ctx.send_event(WindowEvent::Click {
                    x: (lp & 0xFFFF) as i32,
                    y: ((lp >> 16) & 0xFFFF) as i32,
                    state: MouseState::Down,
                    button: MouseButton::Middle,
                });
                return 0;
            }

            WM_RBUTTONDOWN => {
                ctx.send_event(WindowEvent::Click {
                    x: (lp & 0xFFFF) as i32,
                    y: ((lp >> 16) & 0xFFFF) as i32,
                    state: MouseState::Down,
                    button: MouseButton::Right,
                });
                return 0;
            }

            WM_LBUTTONUP => {
                ctx.send_event(WindowEvent::Click {
                    x: (lp & 0xFFFF) as i32,
                    y: ((lp >> 16) & 0xFFFF) as i32,
                    state: MouseState::Up,
                    button: MouseButton::Left,
                });
                return 0;
            }

            WM_MBUTTONUP => {
                ctx.send_event(WindowEvent::Click {
                    x: (lp & 0xFFFF) as i32,
                    y: ((lp >> 16) & 0xFFFF) as i32,
                    state: MouseState::Up,
                    button: MouseButton::Middle,
                });
                return 0;
            }

            WM_RBUTTONUP => {
                ctx.send_event(WindowEvent::Click {
                    x: (lp & 0xFFFF) as i32,
                    y: ((lp >> 16) & 0xFFFF) as i32,
                    state: MouseState::Up,
                    button: MouseButton::Right,
                });
                return 0;
            }

            WM_MOUSEMOVE => {
                ctx.send_event(WindowEvent::MouseMove {
                    x: (lp & 0xFFFF) as i32,
                    y: ((lp >> 16) & 0xFFFF) as i32,
                });
                return 0;
            }

            _ => unsafe { DefWindowProcW(h, msg, wp, lp) },
        }
    }
}

pub struct WindowClient {
    events: Arc<Mutex<Events>>,
    res: Receiver<ServerResponse>,
    req: Sender<ClientRequest>,
}

impl WindowClient {
    pub fn new_window(&self, options: WindowOptions) -> Window {
        self.req
            .send(ClientRequest::CreateWindow { options })
            .expect("falhou ao submeter a mensagem de requisição para criação da janela");

        let res = self
            .res
            .recv()
            .expect("falhou em obter resposta da criação da janela");

        if let ServerResponse::WindowCreated { window } = res {
            return window;
        } else {
            panic!("a resposta do servidor não corresponde à requisição.");
        }
    }

    pub fn poll(&self) -> Option<WindowEvent> {
        let mut events = self.events.lock().unwrap();
        if let Some(WindowEvent::Resize { .. }) = &events.resize {
            return events.resize.take();
        }
        return events.queue.pop_front();
    }

    pub fn terminate(&self) {
        self.req.send(ClientRequest::Terminate);
    }
}

/// Contexto usado apenas durante a criação da janela.
struct CreateCtx {
    events: Arc<Mutex<Events>>,
    opt: WindowOptions,
    window: Option<Window>,
}

impl CreateCtx {
    unsafe fn from_lparam<'a>(lp: isize) -> Option<&'a mut CreateCtx> {
        unsafe {
            let lp = lp as *mut CREATESTRUCTW;
            return lp.as_mut().and_then(|ptr| {
                let p2 = ptr.lpCreateParams as *mut CreateCtx;
                p2.as_mut()
            });
        }
    }
}

/// Contexto de execução da janela já criada.
struct RunCtx {
    events: Arc<Mutex<Events>>,
    state: Arc<Mutex<WindowState>>,
}

impl RunCtx {
    unsafe fn from_hwnd<'a>(handle: HWND) -> Option<&'a mut RunCtx> {
        unsafe {
            let data = api::get_user_data(handle) as *mut RunCtx;
            return data.as_mut();
        }
    }

    fn send_event(&mut self, event: WindowEvent) {
        let mut events = self.events.lock().unwrap();

        match &event {
            WindowEvent::Resize { .. } => {
                events.resize = Some(event);
            }

            _ => {
                events.queue.push_back(event);
            }
        }
    }
}
