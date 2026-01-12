mod window;

use crate::window::{MouseState, Window, WindowEvent, WindowOptions, WindowServer};

struct App {
    window_server: WindowServer,
    window: Window,
    is_running: bool,
}

impl App {
    fn new() -> Self {
        let mut window_server = WindowServer::start();
        let window = window_server.new_window(WindowOptions::default());

        Self {
            window_server,
            window,
            is_running: true,
        }
    }

    fn run(&mut self) {
        loop {
            if !self.is_running {
                break;
            }
            self.process_input();
            self.update();
            self.render();
        }
    }

    fn update(&self) {}

    fn render(&self) {}

    fn process_input(&mut self) {
        while let Some(msg) = self.window_server.poll() {
            match msg.event {
                WindowEvent::Close => {
					if msg.window_id == self.window.id() {
						self.is_running = false;
					}
                }

                WindowEvent::MouseMove { .. } => {}

                WindowEvent::Click { x, y, state, .. } => {
					if state == MouseState::Down {
						println!("click em {} {}", x, y);
					}
                }

                WindowEvent::Resize { .. } => {}
            }
        }
    }
}

fn main() {
    let mut app = App::new();
    app.run();
}
