use std::{
	sync::Arc,
	time::{Duration, Instant},
};

use winit::{
	application::ApplicationHandler,
	dpi::{PhysicalSize, Size},
	event::WindowEvent,
	event_loop::ActiveEventLoop,
	window::{Window, WindowAttributes, WindowId},
};

use crate::{
	input::{InputState, InputSystem},
	renderer::{self, RenderSurface, RenderSystem},
	state::{StateSystem, Tick},
};

/// Host mantém o estado global da aplicação.
pub struct Host {
	renderer: RenderSystem,
	surface: Option<RenderSurface>,
	input: InputSystem,
	state: StateSystem,
	clock: Clock,
}

impl Host {
	pub fn new() -> Self {
		Self {
			renderer: RenderSystem::new(),
			input: InputSystem::new(),
			state: StateSystem::new(),
			clock: Clock::new(),
			surface: None,
		}
	}

	fn redraw(&mut self) {
		let mut tick = Tick::default();
		(tick.delta, tick.elapsed) = self.clock.tock();

		self.state.update(tick, self.input.current_state());

		if let Some(surface) = &self.surface {
			self.renderer.draw(surface);
		}
	}
	
	fn resize(&mut self, width: u32, height: u32) {
		if let Some(surface) = &mut self.surface {
			self.renderer.configure_size(surface, width, height);
		}
	}
}

impl ApplicationHandler for Host {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let attr = Window::default_attributes()
			.with_title("Fever")
			.with_maximized(true);

		let window: Arc<Window> = event_loop
			.create_window(attr)
			.expect("falhou em criar a janela principal.")
			.into();

		self.surface = Some(self.renderer.create_surface(window.clone()));
	}

	fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, ev: WindowEvent) {
		use winit::event::WindowEvent::*;

		#[rustfmt::skip]
		match ev {
			KeyboardInput { event, .. }      => self.input.update_keyboard(event),
			MouseInput { state, button, .. } => self.input.update_mouse(state, button),
			RedrawRequested                  => self.redraw(),
			CloseRequested                   => el.exit(),
			Resized(size)                    => self.resize(size.width, size.height),
			_ => {}
		};
	}
}

struct Clock {
	start: Instant,
	last: Instant,
}

impl Clock {
	fn new() -> Self {
		let now = Instant::now();

		Self {
			start: now,
			last: now,
		}
	}

	fn tock(&mut self) -> (f32, f64) {
		let delta = self.last.elapsed().as_secs_f32();
		let elapsed = self.start.elapsed().as_secs_f64();
		self.last = Instant::now();

		return (delta, elapsed);
	}
}
