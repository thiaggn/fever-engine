use std::collections::HashSet;

use winit::{
	event::{ElementState, KeyEvent, MouseButton, WindowEvent},
	keyboard::{KeyCode, PhysicalKey},
};

use winit::event::ElementState::*;
use winit::event::WindowEvent::*;

#[derive(Default)]
pub struct InputState {
	pub keys: KeyboardState,
	pub mouse: MouseState,
}

pub struct InputSystem {
	state: InputState,
}

impl InputSystem {
	pub fn new() -> Self {
		Self {
			state: InputState::default(),
		}
	}

	pub fn current_state(&self) -> &InputState {
		&self.state
	}

	pub fn update_mouse(&mut self, state: ElementState, button: MouseButton) {
		match state {
			Pressed => self.state.mouse.down.insert(button),
			Released => self.state.mouse.up.insert(button),
		};
	}

	pub fn update_keyboard(&mut self, event: KeyEvent) {
		if let PhysicalKey::Code(code) = event.physical_key {
			match event.state {
				Pressed => self.state.keys.down.insert(code),
				Released => self.state.keys.up.insert(code),
			};
		}
	}

	pub fn reset(&mut self) {
		self.state.keys.down.clear();
		self.state.keys.up.clear();
		self.state.mouse.up.clear();
		self.state.mouse.down.clear();
	}
}

#[derive(Default)]
pub struct KeyboardState {
	down: HashSet<KeyCode>,
	up: HashSet<KeyCode>,
	held: HashSet<KeyCode>,
}

impl KeyboardState {
	pub fn is_up(&self, key: KeyCode) -> bool {
		self.up.contains(&key)
	}

	pub fn is_down(&self, key: KeyCode) -> bool {
		self.down.contains(&key)
	}

	pub fn is_held(&self, key: KeyCode) -> bool {
		self.held.contains(&key)
	}
}

#[derive(Default)]
pub struct MouseState {
	position: (u32, u32),
	delta: (f32, f32),
	held: HashSet<MouseButton>,
	up: HashSet<MouseButton>,
	down: HashSet<MouseButton>,
}

impl MouseState {
	pub fn is_up(&self, button: MouseButton) {
		self.up.contains(&button);
	}

	pub fn is_down(&self, button: MouseButton) {
		self.down.contains(&button);
	}

	pub fn is_held(&self, button: MouseButton) {
		self.held.contains(&button);
	}
}
