use crate::input::InputState;

#[derive(Default)]
pub struct Tick {
	pub elapsed: f64,
	pub delta: f32,
}


pub struct StateSystem {

}

impl StateSystem {
	pub fn new() -> Self {
		Self {  }
	}

	pub fn update(&mut self, tick: Tick, input: &InputState) {

	}

}