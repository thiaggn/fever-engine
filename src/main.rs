#![allow(unused)]

mod cube;
mod host;
mod input;
mod renderer;
mod state;
mod math;

use crate::host::Host;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
	if std::arch::is_x86_feature_detected!("sse4.2") == false {
		panic!("fever: a engine necessita de suporte a SSE4.2");
	}
	
	let mut host = Host::new();

	let event_loop = EventLoop::new().unwrap();
	event_loop.set_control_flow(ControlFlow::Poll);

	event_loop
		.run_app(&mut host)
		.expect("winit: o loop de eventos não terminou graciosamente.");
}
