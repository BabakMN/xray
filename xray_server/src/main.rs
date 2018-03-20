mod app;
mod json_lines_codec;
mod messages;
mod project;
mod window;
mod workspace;

extern crate bytes;
extern crate futures;
extern crate ignore;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_process;
extern crate tokio_uds;
extern crate xray_core;

use std::env;
use std::fs;
use futures::Stream;
use tokio_core::reactor::Core;
use tokio_io::AsyncRead;
use tokio_uds::UnixListener;
use json_lines_codec::JsonLinesCodec;
use messages::{IncomingMessage, OutgoingMessage};
use app::App;

fn main() {
    let socket_path =
        env::var("XRAY_SOCKET_PATH").expect("Missing XRAY_SOCKET_PATH environment variable");

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let mut app = App::new(handle.clone());

    let _ = fs::remove_file(&socket_path);
    let listener = UnixListener::bind(socket_path, &handle).unwrap();

    let handle_connections = listener.incoming().for_each(move |(socket, _)| {
        let framed_socket =
            socket.framed(JsonLinesCodec::<IncomingMessage, OutgoingMessage>::new());
        app.add_connection(framed_socket);
        Ok(())
    });

    println!("Listening");
    core.run(handle_connections).unwrap();
}
