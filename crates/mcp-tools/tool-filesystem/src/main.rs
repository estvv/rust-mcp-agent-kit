mod tools;

use mcp_core::Server;

fn main() {
    let server = Server::new();
    server.run();
}