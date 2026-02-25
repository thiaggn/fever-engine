#![allow(unused)]

use winit::event_loop::{ControlFlow, EventLoop};

use crate::host::Host;

mod host;
mod input;
mod renderer;
mod state;

fn main() {
	let event_loop = EventLoop::new().unwrap();
	event_loop.set_control_flow(ControlFlow::Poll);

	let mut host = Host::new();
	event_loop.run_app(&mut host);
}
