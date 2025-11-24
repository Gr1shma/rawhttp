use anyhow::Result;
use rawhttp::server::Server;

fn main() -> Result<()> {
    println!("rawhttp Server");

    let server = Server::new("127.0.0.1:8080".to_string());
    server.run()?;

    Ok(())
}
