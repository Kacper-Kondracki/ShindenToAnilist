use std::{
    env,
    net::SocketAddr,
};

use reqwest::Client;
use shinden_to_anilist_grpc::{
    pb::shinden_to_anilist_service_server::ShindenToAnilistServiceServer,
    server::ShindenToAnilist,
};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::{
    EnvFilter,
    fmt,
};

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:45187";
const LISTEN_ADDR_ENV: &str = "SHINDEN_TO_ANILIST_GRPC_LISTEN_ADDR";

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("shinden_to_anilist_grpc=info,tower_http=info"));

    fmt().with_env_filter(filter).init();
}

fn listen_addr_argument() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--listen-addr" {
            return args
                .next()
                .map(Some)
                .ok_or_else(|| "--listen-addr requires an address".into());
        }

        if let Some(addr) = arg.strip_prefix("--listen-addr=") {
            return Ok(Some(addr.to_owned()));
        }
    }

    Ok(None)
}

fn listen_addr() -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let addr = listen_addr_argument()?
        .or_else(|| env::var(LISTEN_ADDR_ENV).ok())
        .unwrap_or_else(|| DEFAULT_LISTEN_ADDR.to_owned());

    Ok(addr.parse()?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let addr = listen_addr()?;
    let listener = TcpListener::bind(addr).await?;
    let addr = listener.local_addr()?;
    let service = ShindenToAnilist::new(Client::new());

    info!(%addr, "ShindenToAnilist gRPC listening");
    println!(r#"{{"event":"ready","addr":"{addr}"}}"#);

    Server::builder()
        .accept_http1(true)
        .layer(CorsLayer::permissive())
        .layer(GrpcWebLayer::new())
        .add_service(ShindenToAnilistServiceServer::new(service))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await?;

    Ok(())
}
