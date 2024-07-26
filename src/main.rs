mod server;
mod shared;

use std::env::args;

fn main() {
    let args = args().collect::<Vec<String>>();
    let app = args.get(1).expect("> invalid app");

    if app == "server" {
        return server::start_server();
    } else if app == "client" {
        todo!();
    }
    
    eprintln!("> invalid app");
}
