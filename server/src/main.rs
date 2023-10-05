use std::net::TcpListener;

use server::app;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", 8080))?;

    app::run(listener)?.await
}
