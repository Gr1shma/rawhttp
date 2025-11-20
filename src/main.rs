use anyhow::Result;
use rawhttp::server::tcp::Server;

fn main() -> Result<()> {
    let server = Server::new("127.0.0.1:8080".to_string());
    server.run()
}
