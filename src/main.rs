use crate::{app::App, window::WindowServer};

mod app;
mod renderer;
mod window;

fn main() {
    let (server, client) = WindowServer::connect();

    let trd = std::thread::spawn(move || {
        let app = App::new(client);
        app.run();
    });

    server.run();
    trd.join().unwrap();
}
