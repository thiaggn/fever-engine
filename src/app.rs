#![allow(unused)]

use std::time::{Duration, Instant};

use crate::{renderer::Renderer, window::{Input, Window, WindowClient, WindowOptions}};

pub struct App {
    client: WindowClient,
    window: Window,
	renderer: Renderer,
    should_close: bool,
}

impl App {
    pub fn new(client: WindowClient) -> Self {
        let window = client.new_window(WindowOptions {
            width: 1366,
            height: 768,
            title: "Fever".into(),
        });

        return Self {
			renderer: Renderer::new(window.handle()),
            window,
            client,
            should_close: false,
        };
    }
}

impl App {
    pub fn run(mut self) {
        let target_fps = Duration::from_secs_f32(1.0 / 60.0);
        let mut last_frame = Instant::now();

        loop {
            if self.should_close {
                break;
            }

            let now = Instant::now();
            let elapsed = now.duration_since(last_frame);
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
        self.client.poll_events(|input| match input {
            Input::Close => {
                println!("fechou!");
                self.should_close = true;
            }

            Input::Resize { width, height } => {
                self.renderer.set_dimensions(width, height);
            }

            #[rustfmt::skip]
			Input::Click { x, y, state, button } => {
				println!("click {x}, {y} {:?} {:?}", state, button);
			}

            Input::MouseMove { .. } => {}
        });
    }

    fn render(&self) {
		self.renderer.render();
	}

    fn update(&self, delta: f32) {}
}
