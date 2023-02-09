use hyper::server::conn::http1;
use hyper::service::service_fn;
use service::preprocess;
use tokio::net::TcpListener;

mod config;
mod service;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = config::Config::load().expect("failed to initialize config");
    let addr = config.server.ip + ":" + &config.server.port.to_string();
    let listener = TcpListener::bind(&addr).await?;
    println!("listen on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("connect to {}", addr);

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(preprocess))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
