use crate::{
    app::App,
    window::{WindowServer},
};

mod app;
mod window;
mod renderer;

fn main() {
    let mut server = WindowServer::new();
    let client = server.client();

    let trd = std::thread::spawn(move || {
        let mut app = App::new(client);
        app.run();
    });

    server.run();
    trd.join().unwrap();
}
