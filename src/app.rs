#![allow(unused)]

use std::time::{Duration, Instant};

use crate::window::{Window, WindowClient, WindowEvent};

pub struct App {
    client: WindowClient,
    window: Window,
    should_close: bool,
    last_frame: Instant,
}

impl App {
    pub fn new(client: WindowClient) -> Self {
        let window = client.new_window(Default::default());
        return Self {
            window,
            client,
            should_close: false,
            last_frame: Instant::now(),
        };
    }

    pub fn run(&mut self) {
        let target_fps = Duration::from_secs_f32(1.0 / 10.0);
        let mut last_frame = Instant::now();

        loop {
            if self.should_close {
                break;
            }

            let now = Instant::now();
            let mut elapsed = now.duration_since(last_frame);
            last_frame = now;

            self.input();
            self.update(elapsed.as_secs_f32());
            self.render();

            if elapsed < target_fps {
                std::thread::sleep(target_fps - elapsed);
            }
        }

        self.client.terminate();
    }

    fn input(&mut self) {
        while let Some(event) = self.client.poll() {
            match event {
                WindowEvent::Close => {
					println!("fechou!");
                    self.should_close = true;
                }

                WindowEvent::Resize { width, height } => {
                    println!("redimensionou: {} {}", width, height);
                }

                WindowEvent::Click {
                    x,
                    y,
                    state,
                    button,
                } => {
                    println!("click {x}, {y} {:?} {:?}", state, button);
                }

                WindowEvent::MouseMove { x, y } => {
					println!("moveu: {x}, {y}")
				}
            }
        }
    }

    fn render(&self) {}

    fn update(&self, delta: f32) {}
}
